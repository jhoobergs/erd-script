use crate::ast::{Attribute, AttributeType, DataType};
use crate::ast::{Expr, ForeignKey, Ident};
use crate::erd::{ERDError, ERD};
use crate::sql::SQL;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt::Write;
use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq)]
pub enum TableDescription {
    Entity(EntityTableDescription),
    Relation(RelationTableDescription),
}

impl TableDescription {
    pub fn to_table(&self, erd: &ERD) -> Table {
        match self {
            TableDescription::Entity(e) => e.to_table(erd),
            TableDescription::Relation(r) => r.to_table(erd),
        }
    }
    pub fn check_entity_or_relation(&self, erd: &ERD) -> bool {
        match self {
            TableDescription::Entity(e) => e.check_entity(erd),
            TableDescription::Relation(r) => r.check_relation(erd),
        }
    }
    pub fn name(&self) -> Ident {
        match self {
            TableDescription::Entity(e) => e.name.clone(),
            TableDescription::Relation(r) => r.name.clone(),
        }
    }
    pub fn er(&self) -> Ident {
        match self {
            TableDescription::Entity(e) => e.entity.clone(),
            TableDescription::Relation(r) => r.relation.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityTableDescription {
    name: Ident,
    entity: Ident,
    foreign_keys: Vec<ForeignKey>,
}

impl EntityTableDescription {
    pub fn to_table(&self, erd: &ERD) -> Table {
        Table {
            name: self.name.clone(),
            columns: erd
                .get_entity_attributes(self.entity.clone())
                .into_iter()
                .chain(self.foreign_keys.iter().flat_map(|c| {
                    let other_member = erd
                        .get_relation(c.relation.to_owned())
                        .unwrap()
                        .find_other_member(self.name.clone());
                    erd.get_entity_ids(other_member)
                        .into_iter()
                        .map(move |a| Attribute {
                            ident: c.attribute_name.clone(),
                            r#type: AttributeType::Normal,
                            datatype: a.get_data_type().map(|d| d.foreign_key_type()),
                        })
                }))
                .map(|c| TableColumn {
                    name: c.get_ident(),
                    datatype: c.get_data_type().unwrap(),
                })
                .collect(),
            primary_key_parts: erd
                .get_entity_ids(self.entity.clone())
                .into_iter()
                .map(|a| a.get_ident())
                .collect(),
        }
    }
    pub fn check_entity(&self, erd: &ERD) -> bool {
        erd.has_entity(self.entity.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationTableDescription {
    name: Ident,
    relation: Ident, // entity or relation where it comes from
}

impl RelationTableDescription {
    pub fn to_table(&self, erd: &ERD) -> Table {
        let relation = erd.get_relation(self.relation.clone()).unwrap();
        let members = relation.get_members();
        let mut used_members: HashSet<Ident> = HashSet::new();
        let primary_key_parts: Vec<_> = members
            .iter()
            .flat_map(|e| {
                let postfix = if used_members.contains(&e) {
                    "2" // TODO improve if other than binary relation are supported
                } else {
                    used_members.insert(e.to_owned());
                    ""
                };
                erd.get_entity_ids(e.to_owned()).into_iter().map(move |a| {
                    a.renamed(format!("{}_{}{}", e, a.ident.to_string(), postfix).into())
                })
            })
            .collect();

        Table {
            name: self.name.clone(),
            columns: erd
                .get_relation_attributes(self.relation.clone())
                .into_iter()
                .chain(primary_key_parts.clone().into_iter())
                .map(|c| TableColumn {
                    name: c.get_ident(),
                    datatype: c.get_data_type().unwrap().foreign_key_type(),
                })
                .collect(),
            primary_key_parts: primary_key_parts
                .into_iter()
                .map(|a| a.get_ident())
                .collect(),
        }
    }
    pub fn check_relation(&self, erd: &ERD) -> bool {
        erd.has_relation(self.relation.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    name: Ident,
    datatype: DataType,
    // TODO unique, nullable, type ...
}

impl TableColumn {
    fn write_sql_create_lines(&self, s: &mut String, sql: SQL) {
        write!(
            s,
            "{} {},",
            sql.to_column_ident(&self.name),
            sql.to_data_type(&self.datatype)
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    name: Ident,
    columns: Vec<TableColumn>,
    primary_key_parts: Vec<Ident>,
}

impl Table {
    fn write_sql_create(&self, s: &mut String, sql: SQL) {
        write!(s, "CREATE TABLE {} (\n", self.name);
        for col in self.columns.iter() {
            col.write_sql_create_lines(s, sql);
            write!(s, "\n");
        }
        write!(
            s,
            "PRIMARY KEY ({})\n",
            self.primary_key_parts
                .iter()
                .map(|p| sql.to_column_ident(p))
                .collect::<Vec<_>>()
                .join(","),
        );
        write!(s, ");");
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    ForeignKey(ForeignKeyConstraint),
}

impl Constraint {
    fn write_sql_create(&self, s: &mut String, sql: SQL) {
        match self {
            Self::ForeignKey(f) => f.write_sql_create(s, sql),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForeignKeyConstraint {
    table_name: Ident,
    column_names: Vec<Ident>,
    other_table_name: Ident,
    other_table_column_names: Vec<Ident>,
}

impl ForeignKeyConstraint {
    fn write_sql_create(&self, s: &mut String, sql: SQL) {
        write!(
            s,
            "ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {}({});",
            self.table_name,
            self.column_names
                .iter()
                .map(|a| sql.to_column_ident(a))
                .collect::<Vec<_>>()
                .join(","),
            self.other_table_name,
            self.other_table_column_names
                .iter()
                .map(|a| sql.to_column_ident(a))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Physical {
    tables: Vec<Table>,
    constraints: Vec<Constraint>,
}

impl Physical {
    pub fn write_sql_create(&self, s: &mut String, sql: SQL) {
        for col in self.tables.iter() {
            col.write_sql_create(s, sql);
            write!(s, "\n");
        }
        for constraint in self.constraints.iter() {
            constraint.write_sql_create(s, sql);
            write!(s, "\n");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalDescription {
    erd: ERD,
    tables: Vec<TableDescription>,
}

impl PhysicalDescription {
    pub fn to_dot(&self) -> crate::dot::Graph {
        self.erd.to_dot()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalFromScriptError {
    ParsingError(crate::parser::ConsumeError),
    ERDError(Vec<ERDError>),
    PhysicalError(Vec<PhysicalError>),
}

impl PhysicalDescription {
    pub fn from_script(content: &str) -> Result<Self, PhysicalFromScriptError> {
        let pairs = crate::parser::parse_as_erd(&content).map_err(|e| {
            PhysicalFromScriptError::ParsingError(crate::parser::ConsumeError::ERDParseError(vec![
                e,
            ]))
        })?;
        let asts = crate::parser::consume_expressions(pairs)
            .map_err(PhysicalFromScriptError::ParsingError)?;
        let res: Result<Self, PhysicalERDError> = asts.try_into();
        res.map_err(|e| e.into())
    }
}

impl PhysicalDescription {
    fn validate(&self) -> Vec<PhysicalError> {
        let mut errors = Vec::new();
        let mut converted_entities_relations: HashSet<Ident> = HashSet::new();
        let mut table_names: HashSet<Ident> = HashSet::new();
        for t in self.tables.iter() {
            if table_names.contains(&t.name()) {
                errors.push(PhysicalError::DuplicateTableName(t.name().clone()));
            } else {
                table_names.insert(t.name().clone());
            }
            if !t.check_entity_or_relation(&self.erd) {
                errors.push(PhysicalError::UnknownEntityOrRelationInTable(
                    t.er(),
                    t.name(),
                ))
            } else if converted_entities_relations.contains(&t.er()) {
                errors.push(PhysicalError::ConvertedMoreThanOnce(t.er()));
            } else {
                converted_entities_relations.insert(t.er());
                if let TableDescription::Entity(et) = t {
                    let mut column_names: HashSet<Ident> = HashSet::from_iter(
                        self.erd
                            .get_entity_attributes(t.er())
                            .into_iter()
                            .map(|a| a.get_ident()),
                    );
                    for foreign_key in et.foreign_keys.iter() {
                        if column_names.contains(&foreign_key.attribute_name) {
                            errors.push(PhysicalError::DuplicateColumnNameInTable(
                                foreign_key.attribute_name.clone(),
                                t.name(),
                            ));
                        } else {
                            column_names.insert(foreign_key.attribute_name.clone());
                        }
                        let relation = self.erd.get_relation(foreign_key.relation.clone());
                        if let Some(r) = relation {
                            if r.degree() != 2 {
                                errors.push(PhysicalError::UnsupportedRelationDegree(r.name()));
                            } else if r.can_work_with_foreign_key(foreign_key.relation.clone()) {
                                if converted_entities_relations.contains(&foreign_key.relation) {
                                    errors.push(PhysicalError::ConvertedMoreThanOnce(
                                        foreign_key.relation.clone(),
                                    ));
                                } else {
                                    converted_entities_relations
                                        .insert(foreign_key.relation.clone());
                                }
                            } else {
                                errors.push(PhysicalError::ImpossibleForeignKey(
                                    foreign_key.relation.clone(),
                                    t.name(),
                                ))
                            }
                        } else {
                            errors.push(PhysicalError::ForeignKeyToEntityInTable(
                                foreign_key.relation.clone(),
                                t.name(),
                            ));
                        }
                    }
                }
            }
        }

        let erd_entities_relations = self.erd.get_idents();
        let forgotten: HashSet<_> = erd_entities_relations
            .difference(&converted_entities_relations)
            .collect();
        for item in forgotten.iter() {
            errors.push(PhysicalError::ForgottenEntityOrRelation(
                item.clone().clone(),
            ));
        }

        errors
    }
    pub fn to_physical(&self) -> Physical {
        let mut tables: Vec<Table> = Vec::new();
        let mut constraints: Vec<Constraint> = Vec::new();

        let mut entity_name_to_table_name: HashMap<Ident, Ident> = HashMap::new();
        for t in self.tables.iter() {
            if let TableDescription::Entity(e) = t {
                entity_name_to_table_name.insert(e.entity.clone(), e.name.clone());
            }
        }

        for t in self.tables.iter() {
            tables.push(t.to_table(&self.erd));

            match t {
                TableDescription::Entity(et) => {
                    for foreign_key in et.foreign_keys.iter() {
                        let other_entity = self
                            .erd
                            .get_relation(foreign_key.relation.clone())
                            .unwrap()
                            .find_other_member(t.name()); // TODO renamings? & more than degree 2
                        constraints.push(Constraint::ForeignKey(ForeignKeyConstraint {
                            table_name: t.name(),
                            column_names: vec![foreign_key.attribute_name.clone()],
                            other_table_name: entity_name_to_table_name
                                .get(&other_entity)
                                .unwrap()
                                .to_owned(),
                            other_table_column_names: self
                                .erd
                                .get_entity_ids(other_entity.clone())
                                .into_iter()
                                .map(|a| a.get_ident())
                                .collect(),
                        }));
                    }
                }
                TableDescription::Relation(r) => {
                    let relation = self.erd.get_relation(r.relation.clone()).unwrap();
                    let mut used_members: HashSet<Ident> = HashSet::new(); // TODO: reduce duplication?
                    for member in relation.get_members().iter() {
                        let other_table_column_names: Vec<_> = self
                            .erd
                            .get_entity_ids(member.clone())
                            .into_iter()
                            .map(|a| a.get_ident().clone())
                            .collect();

                        let postfix = if used_members.contains(&member) {
                            "2" // TODO improve if other than binary relation are supported
                        } else {
                            used_members.insert(member.to_owned());
                            ""
                        };
                        constraints.push(Constraint::ForeignKey(ForeignKeyConstraint {
                            table_name: t.name(),
                            other_table_name: entity_name_to_table_name
                                .get(&member)
                                .unwrap()
                                .to_owned(),
                            column_names: other_table_column_names
                                .iter()
                                .map(|n| format!("{}_{}{}", member, n, postfix).into())
                                .collect(),
                            other_table_column_names,
                        }));
                    }
                }
            }
        }

        Physical {
            tables,
            constraints,
        }
    }
}

impl std::convert::TryFrom<Vec<Expr>> for PhysicalDescription {
    type Error = PhysicalERDError;
    fn try_from(v: Vec<Expr>) -> Result<Self, Self::Error> {
        let erd: ERD = v.clone().try_into().map_err(PhysicalERDError::ERD)?;
        let tables = v
            .iter()
            .filter_map(|expr| match expr {
                Expr::EntityTable(name, entity, foreign_keys) => {
                    Some(TableDescription::Entity(EntityTableDescription {
                        name: name.clone(),
                        entity: entity.clone(),
                        foreign_keys: foreign_keys.clone(),
                    }))
                }
                Expr::RelationTable(name, relation) => {
                    Some(TableDescription::Relation(RelationTableDescription {
                        name: name.clone(),
                        relation: relation.clone(),
                    }))
                }
                _ => None,
            })
            .collect();

        let p = PhysicalDescription { erd, tables };

        let validation = p.validate();
        if validation.is_empty() {
            Ok(p)
        } else {
            Err(PhysicalERDError::Physical(validation))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PhysicalERDError {
    ERD(Vec<ERDError>),
    Physical(Vec<PhysicalError>),
}

impl std::convert::From<PhysicalERDError> for PhysicalFromScriptError {
    fn from(v: PhysicalERDError) -> Self {
        match v {
            PhysicalERDError::ERD(v) => Self::ERDError(v),
            PhysicalERDError::Physical(v) => Self::PhysicalError(v),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PhysicalError {
    DuplicateTableName(Ident),
    DuplicateColumnNameInTable(Ident, Ident), // Column, Table
    UnknownEntityOrRelationInTable(Ident, Ident), // EntityRelation, Table
    ConvertedMoreThanOnce(Ident),             // Relation / Entity
    ForgottenEntityOrRelation(Ident),
    UnsupportedRelationDegree(Ident),
    ForeignKeyToEntityInTable(Ident, Ident), // Entity, Table
    ImpossibleForeignKey(Ident, Ident),      // Entity, Table
}

impl std::fmt::Display for PhysicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateTableName(i) => write!(f, "Table name {} is used multiple times.", i),
            Self::DuplicateColumnNameInTable(i, e) => {
                write!(f, "Multiple columns named {} in table {}.", i, e)
            }
            Self::UnknownEntityOrRelationInTable(e, r) => {
                write!(f, "Unknown entity or relation {} in table {}.", e, r)
            }
            Self::ConvertedMoreThanOnce(e) => {
                write!(
                    f,
                    "Relation or entity {} is converted more than once to a table.",
                    e
                )
            }
            Self::ForgottenEntityOrRelation(e) => {
                write!(
                    f,
                    "Relation or entity {} is not converted to the physical representation.",
                    e
                )
            }
            Self::UnsupportedRelationDegree(e) => {
                write!(
                    f,
                    "Relation {} has a degree different from 2 and that is (currently) not supported. Please convert the relation to relations of degree 2.",
                    e
                )
            }
            Self::ImpossibleForeignKey(e, t) => {
                write!(
                    f,
                    "It is impossible to create table {} with foreign key {}.",
                    t, e
                )
            }
            Self::ForeignKeyToEntityInTable(e, t) => {
                write!(
                    f,
                    "You created a foreign key to the entity {} in table {} instead of to a relation.",
                    e, t
                )
            }
        }
    }
}
