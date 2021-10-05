use crate::ast::{Expr, ForeignKey, Ident};
use crate::erd::{ERDError, ERD};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq)]
pub struct TableDescription {
    name: Ident,
    er: Ident, // entity or relation where it comes from
    foreign_keys: Vec<ForeignKey>,
}

impl TableDescription {
    pub fn to_table(&self, erd: &ERD) -> Table {
        Table {
            name: self.name.clone(),
            columns: erd
                .get_attributes(self.er.clone())
                .into_iter()
                .map(|c| c.get_ident())
                .chain(self.foreign_keys.iter().map(|c| c.attribute_name.clone()))
                .map(|name| TableColumn { name: name.clone() })
                .collect(),
            primary_key_parts: erd
                .get_attributes(self.er.clone())
                .into_iter()
                .filter_map(|c| match c {
                    crate::ast::Attribute::Normal(_) => None,
                    crate::ast::Attribute::Key(k) => Some(k),
                })
                .collect(),
            constraints: self
                .foreign_keys
                .iter()
                .map(|c| {
                    Constraint::ForeignKey(ForeignKeyConstraint {
                        column_name: c.attribute_name.clone(),
                        other_table_name: c.relation.clone(), // TODO renamings?
                        other_table_column: "test".to_string().into(), // TODO
                    })
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    name: Ident,
    // TODO unique, nullable ...
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    name: Ident,
    columns: Vec<TableColumn>,
    primary_key_parts: Vec<Ident>,
    constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    ForeignKey(ForeignKeyConstraint),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForeignKeyConstraint {
    column_name: Ident,
    other_table_name: Ident,
    other_table_column: Ident,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Physical {
    erd: ERD,
    tables: Vec<TableDescription>,
}

impl Physical {
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

impl Physical {
    pub fn from_script(content: &str) -> Result<Physical, PhysicalFromScriptError> {
        let pairs = crate::parser::parse_as_erd(&content).map_err(|e| {
            PhysicalFromScriptError::ParsingError(crate::parser::ConsumeError::ERDParseError(vec![
                e,
            ]))
        })?;
        let asts = crate::parser::consume_expressions(pairs)
            .map_err(PhysicalFromScriptError::ParsingError)?;
        let res: Result<Physical, PhysicalERDError> = asts.try_into();
        res.map_err(|e| e.into())
    }
}

impl Physical {
    fn validate(&self) -> Vec<PhysicalError> {
        let mut errors = Vec::new();
        let mut converted_entities_relations: HashSet<Ident> = HashSet::new();
        let mut table_names: HashSet<Ident> = HashSet::new();
        for t in self.tables.iter() {
            if table_names.contains(&t.name) {
                errors.push(PhysicalError::DuplicateTableName(t.name.clone()));
            } else {
                table_names.insert(t.name.clone());
            }
            if !self.erd.has_entity_or_relation(t.er.clone()) {
                errors.push(PhysicalError::UnknownEntityOrRelationInTable(
                    t.er.clone(),
                    t.name.clone(),
                ))
            } else if converted_entities_relations.contains(&t.er) {
                errors.push(PhysicalError::ConvertedMoreThanOnce(t.er.clone()));
            } else {
                converted_entities_relations.insert(t.er.clone());
                let mut column_names: HashSet<Ident> = HashSet::from_iter(
                    self.erd
                        .get_attributes(t.er.clone())
                        .into_iter()
                        .map(|a| a.get_ident()),
                );
                for foreign_key in t.foreign_keys.iter() {
                    if column_names.contains(&foreign_key.attribute_name) {
                        errors.push(PhysicalError::DuplicateColumnNameInTable(
                            foreign_key.attribute_name.clone(),
                            t.name.clone(),
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
                                converted_entities_relations.insert(foreign_key.relation.clone());
                            }
                        } else {
                            errors.push(PhysicalError::ImpossibleForeignKey(
                                foreign_key.relation.clone(),
                                t.name.clone(),
                            ))
                        }
                    } else {
                        errors.push(PhysicalError::ForeignKeyToEntityInTable(
                            foreign_key.relation.clone(),
                            t.name.clone(),
                        ));
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
    pub fn to_tables(&self) -> Vec<Table> {
        let mut tables: Vec<Table> = Vec::new();
        for t in self.tables.iter() {
            tables.push(t.to_table(&self.erd));
        }
        tables
    }
}

impl std::convert::TryFrom<Vec<Expr>> for Physical {
    type Error = PhysicalERDError;
    fn try_from(v: Vec<Expr>) -> Result<Self, Self::Error> {
        let erd: ERD = v.clone().try_into().map_err(PhysicalERDError::ERD)?;
        let tables = v
            .iter()
            .filter_map(|expr| match expr {
                Expr::Table(name, er, foreign_keys) => Some(TableDescription {
                    name: name.clone(),
                    er: er.clone(),
                    foreign_keys: foreign_keys.clone(),
                }),
                _ => None,
            })
            .collect();

        let p = Physical { erd, tables };

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
