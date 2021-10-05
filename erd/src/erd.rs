use crate::ast::{Attribute, Expr, Ident, RelationMember, RelationOptionality};
use crate::dot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERD {
    entities: Vec<Entity>,
    relations: Vec<Relation>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ERDFromScriptError {
    ParsingError(crate::parser::ConsumeError),
    ERDError(Vec<ERDError>),
}

impl ERD {
    pub fn from_script(content: &str) -> Result<ERD, ERDFromScriptError> {
        let pairs = crate::parser::parse_as_erd(&content).map_err(|e| {
            ERDFromScriptError::ParsingError(crate::parser::ConsumeError::ERDParseError(vec![e]))
        })?;
        let asts =
            crate::parser::consume_expressions(pairs).map_err(ERDFromScriptError::ParsingError)?;
        asts.try_into().map_err(ERDFromScriptError::ERDError)
    }
}

impl ERD {
    fn validate(&self) -> Vec<ERDError> {
        let mut errors = Vec::new();
        let mut entity_names: HashSet<Ident> = HashSet::new();
        for e in self.entities.iter() {
            if entity_names.contains(&e.name) {
                errors.push(ERDError::DuplicateIdent(e.name.clone()))
            } else {
                entity_names.insert(e.name.clone());
            }
            let mut entity_attributes = HashSet::new();
            for attribute in e.attributes.iter() {
                if entity_attributes.contains(attribute) {
                    errors.push(ERDError::DuplicateAttributeInEntity(
                        attribute.get_ident(),
                        e.name.clone(),
                    ));
                } else {
                    entity_attributes.insert(attribute.clone());
                }
            }
        }

        let mut relation_names: HashSet<Ident> = HashSet::new();
        for r in self.relations.iter() {
            if entity_names.contains(&r.name) || relation_names.contains(&r.name) {
                errors.push(ERDError::DuplicateIdent(r.name.clone()))
            } else {
                relation_names.insert(r.name.clone());
            }
            for member in r.members.iter() {
                if !entity_names.contains(&member.entity) {
                    errors.push(ERDError::UnknownEntityInRelation(
                        member.entity.clone(),
                        r.name.clone(),
                    ));
                }
            }
            let mut relation_attributes = HashSet::new();
            for attribute in r.attributes.iter() {
                if relation_attributes.contains(attribute) {
                    errors.push(ERDError::DuplicateAttributeInRelation(
                        attribute.get_ident(),
                        r.name.clone(),
                    ));
                } else {
                    relation_attributes.insert(attribute.clone());
                }
            }
        }

        errors
    }
}

impl ERD {
    pub fn has_entity_or_relation(&self, name: Ident) -> bool {
        self.entities.iter().find(|e| e.name == name).is_some()
            || self.relations.iter().find(|e| e.name == name).is_some()
    }

    pub fn get_attributes(&self, name: Ident) -> Vec<Attribute> {
        if let Some(e) = self.entities.iter().find(|e| e.name == name) {
            e.attributes.clone()
        } else if let Some(r) = self.relations.iter().find(|r| r.name == name) {
            r.attributes.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_idents(&self) -> HashSet<Ident> {
        self.entities
            .iter()
            .map(|e| e.name.clone())
            .chain(self.relations.iter().map(|e| e.name.clone()))
            .collect()
    }

    pub fn get_relation(&self, name: Ident) -> Option<Relation> {
        self.relations
            .iter()
            .find(|e| e.name == name)
            .map(|a| a.to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    name: Ident,
    attributes: Vec<Attribute>,
}

impl Attribute {
    fn to_dot_statements(&self, entity: Ident) -> Vec<dot::Statement> {
        let mut attributes = vec![dot::AListItem {
            key: "shape".into(),
            value: "ellipse".into(),
        }];

        let attribute_name: String = self.get_ident().into();
        attributes.push(dot::AListItem {
            key: "label".into(),
            value: if let Attribute::Key(_) = self {
                format!("<<U>{}</U>>", attribute_name)
            } else {
                attribute_name.clone()
            },
        });
        let entity_name: String = entity.into();
        vec![dot::Statement::Node(dot::NodeStatement {
            node: format!("{}_{}", entity_name, attribute_name),
            attributes: Some(dot::AttributeList {
                content: dot::AList(attributes),
                tail: Box::new(None),
            }),
        })]
    }
}

impl ToDot for Entity {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let entity_node = dot::Statement::Node(dot::NodeStatement {
            node: self.name.clone().into(),
            attributes: Some(dot::AttributeList {
                content: dot::AList(vec![dot::AListItem {
                    key: "shape".into(),
                    value: "box".into(),
                }]),
                tail: Box::new(None),
            }),
        });

        let mut statements = vec![entity_node];

        // draw attributes
        statements.extend(
            self.attributes
                .iter()
                .flat_map(|a| a.to_dot_statements(self.name.clone())),
        );

        let entity_name: String = self.name.clone().into();
        // Draw attribute lines
        for attribute in self.attributes.iter() {
            let attribute_name: String = attribute.get_ident().into();
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: self.name.clone().into(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Normal,
                    id: format!("{}_{}", entity_name, attribute_name),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![dot::AListItem {
                        key: "len".into(),
                        value: "1.00".to_string(),
                    }]),
                    tail: Box::new(None),
                }),
            }));
        }
        statements
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    name: Ident,
    label: Option<String>,
    members: Vec<RelationMember>,
    attributes: Vec<Attribute>,
}

impl Relation {
    pub fn name(&self) -> Ident {
        return self.name.clone();
    }

    pub fn degree(&self) -> usize {
        self.members.len()
    }

    pub fn can_work_with_foreign_key(&self, entity: Ident) -> bool {
        let mut nb_more_than_one = 0;
        for member in self.members.iter() {
            match member.cardinality {
                crate::ast::RelationCardinality::One => (),
                _ => nb_more_than_one += 1,
            }
        }
        self.degree() == 2
            && nb_more_than_one < 2
            && self
                .members
                .iter()
                .find(|m| m.entity != entity)
                .map(|m| m.cardinality == crate::ast::RelationCardinality::One)
                .unwrap_or(false)
    }
}

impl ToDot for Relation {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let relation_name: String = self.name.clone().into();

        let relation_label = if let Some(l) = self.label.clone() {
            l
        } else {
            relation_name.clone()
        };
        let relation_node = dot::Statement::Node(dot::NodeStatement {
            node: relation_name.clone(),
            attributes: Some(dot::AttributeList {
                content: dot::AList(vec![
                    dot::AListItem {
                        key: "shape".into(),
                        value: "diamond".into(),
                    },
                    dot::AListItem {
                        key: "label".into(),
                        value: format!("\"{}\"", relation_label),
                    },
                ]),
                tail: Box::new(None),
            }),
        });

        let mut statements = vec![relation_node];

        // draw attributes
        statements.extend(
            self.attributes
                .iter()
                .flat_map(|a| a.to_dot_statements(self.name.clone())),
        );

        // Draw attribute lines
        for attribute in self.attributes.iter() {
            let attribute_name: String = attribute.get_ident().into();
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: self.name.clone().into(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Normal,
                    id: format!("{}_{}", relation_name, attribute_name),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![dot::AListItem {
                        key: "len".into(),
                        value: "1.00".to_string(),
                    }]),
                    tail: Box::new(None),
                }),
            }));
        }

        // Draw member lines
        let mut next_multiple_amount: char = 'n';
        for member in self.members.iter() {
            let (amount, new_next_multiple_amount) =
                member.cardinality.get_amount(next_multiple_amount);
            next_multiple_amount = new_next_multiple_amount;
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: self.name.clone().into(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Normal,
                    id: member.entity.clone().into(),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![
                        dot::AListItem {
                            key: "color".into(),
                            value: match member.optionality {
                                RelationOptionality::Optional => "black",
                                RelationOptionality::Required => "\"black:invis:invis:black\"",
                            }
                            .into(),
                        },
                        dot::AListItem {
                            key: "label".into(),
                            value: format!("<<font color=\"blue\">{}</font>>", amount),
                        },
                        dot::AListItem {
                            key: "len".into(),
                            value: "1.00".to_string(),
                        },
                    ]),
                    tail: Box::new(None),
                }),
            }));
        }
        statements
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ERDError {
    DuplicateIdent(Ident),
    DuplicateAttributeInEntity(Ident, Ident), // Attribute, Entity
    DuplicateAttributeInRelation(Ident, Ident), // Attribute, Relation
    UnknownEntityInRelation(Ident, Ident),    // Entity, Relation
}
impl std::fmt::Display for ERDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateIdent(i) => write!(f, "Name {} is used multiple times.", i),
            Self::DuplicateAttributeInEntity(i, e) => {
                write!(f, "Multiple attributes named {} in entity {}.", i, e)
            }
            Self::DuplicateAttributeInRelation(i, e) => {
                write!(f, "Multiple attributes named {} in relation {}.", i, e)
            }
            Self::UnknownEntityInRelation(e, r) => {
                write!(f, "Unknown entity {} in relation {}.", e, r)
            }
        }
    }
}

impl std::convert::TryFrom<Vec<Expr>> for ERD {
    type Error = Vec<ERDError>;
    fn try_from(v: Vec<Expr>) -> Result<ERD, Vec<ERDError>> {
        let entities = v
            .iter()
            .filter_map(|expr| match expr {
                Expr::Entity(name, attributes) => Some(Entity {
                    name: name.clone(),
                    attributes: attributes.clone(),
                }),
                _ => None,
            })
            .collect();
        let relations = v
            .iter()
            .filter_map(|expr| match expr {
                Expr::Relation(name, label, members, attributes) => Some(Relation {
                    name: name.clone(),
                    label: label.clone(),
                    members: members.clone(),
                    attributes: attributes.clone(), // TODO
                }),
                _ => None,
            })
            .collect();
        let erd = ERD {
            entities,
            relations,
        };
        let validation = erd.validate();
        if validation.is_empty() {
            Ok(erd)
        } else {
            Err(validation)
        }
    }
}

pub trait ToDot {
    fn to_dot_statements(&self) -> Vec<dot::Statement>;
}

impl<T: ToDot> ToDot for Vec<T> {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        self.iter().flat_map(|e| e.to_dot_statements()).collect()
    }
}

impl ToDot for ERD {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let mut statements = Vec::new();

        statements.push(dot::Statement::ID("layout".into(), "neato".into()));
        statements.push(dot::Statement::ID("forcelabels".into(), "true".into()));
        statements.push(dot::Statement::ID("overlap".into(), "scale".into()));

        statements.push(dot::Statement::Attribute(dot::AttributeStatement {
            r#type: dot::AttributeStatementType::Graph,
            attributes: dot::AttributeList {
                content: dot::AList(vec![
                    dot::AListItem {
                        key: "pad".into(),
                        value: "2".to_string(),
                    },
                    dot::AListItem {
                        key: "nodesep".into(),
                        value: "1".to_string(),
                    },
                    dot::AListItem {
                        key: "ranksep".into(),
                        value: "2".to_string(),
                    },
                ]),
                tail: Box::new(None),
            },
        }));

        statements.extend(self.entities.to_dot_statements());
        statements.extend(self.relations.to_dot_statements());

        statements
    }
}

impl ERD {
    pub fn to_dot(&self) -> dot::Graph {
        dot::Graph {
            strict: false,
            r#type: dot::GraphType::Normal,
            id: None, // TODO
            statements: self.to_dot_statements(),
        }
    }
}
