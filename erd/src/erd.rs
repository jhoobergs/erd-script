use crate::ast::{Attribute, Expr, Ident, RelationMember, RelationOptionality};
use crate::dot;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERD {
    entities: Vec<Entity>,
    relations: Vec<Relation>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    name: Ident,
    attributes: Vec<Attribute>,
}

impl ToDot for Attribute {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let mut statements = Vec::new();

        let mut attributes = vec![dot::AListItem {
            key: "shape".into(),
            value: "ellipse".into(),
        }];

        if let Attribute::Key(i) = self {
            let s: String = i.clone().into();
            attributes.push(dot::AListItem {
                key: "label".into(),
                value: format!("<<U>{}</U>>", s),
            });
        }

        statements.push(dot::Statement::Attribute(dot::AttributeStatement {
            r#type: dot::AttributeStatementType::Node,

            attributes: dot::AttributeList {
                content: dot::AList(attributes),
                tail: Box::new(None),
            },
        }));

        statements.push(dot::Statement::Node(dot::NodeStatement {
            node: self.get_ident().into(),
            attributes: None,
        }));

        statements
    }
}

impl ToDot for Entity {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let entity_node_attribute = dot::Statement::Attribute(dot::AttributeStatement {
            r#type: dot::AttributeStatementType::Node,

            attributes: dot::AttributeList {
                content: dot::AList(vec![dot::AListItem {
                    key: "shape".into(),
                    value: "box".into(),
                }]),
                tail: Box::new(None),
            },
        });

        let entity_node = dot::Statement::Node(dot::NodeStatement {
            node: self.name.clone().into(),
            attributes: None,
        });

        let mut statements = vec![entity_node_attribute, entity_node];

        // draw attributes
        statements.extend(self.attributes.to_dot_statements());
        // Draw attribute lines
        for attribute in self.attributes.iter() {
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: self.name.clone().into(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Directional,
                    id: attribute.get_ident().into(),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![dot::AListItem {
                        key: "dir".into(),
                        value: "none".into(),
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

impl ToDot for Relation {
    fn to_dot_statements(&self) -> Vec<dot::Statement> {
        let relation_name: String = self.name.clone().into();
        let relation_node_attribute = dot::Statement::Attribute(dot::AttributeStatement {
            r#type: dot::AttributeStatementType::Node,

            attributes: dot::AttributeList {
                content: dot::AList(vec![
                    dot::AListItem {
                        key: "shape".into(),
                        value: "diamond".into(),
                    },
                    dot::AListItem {
                        key: "label".into(),
                        value: format!(
                            "\"{} ({})\"",
                            relation_name.clone(),
                            self.members
                                .iter()
                                .map(|m| m.cardinality.get_amount())
                                .collect::<Vec<_>>()
                                .join(":")
                        ),
                    },
                ]),
                tail: Box::new(None),
            },
        });

        let relation_node = dot::Statement::Node(dot::NodeStatement {
            node: relation_name.clone(),
            attributes: None,
        });

        let mut statements = vec![relation_node_attribute, relation_node];

        // draw attributes
        statements.extend(self.attributes.to_dot_statements());
        // Draw attribute lines
        for attribute in self.attributes.iter() {
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: relation_name.clone(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Directional,
                    id: attribute.get_ident().into(),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![dot::AListItem {
                        key: "dir".into(),
                        value: "none".into(),
                    }]),
                    tail: Box::new(None),
                }),
            }));
        }

        // Draw member lines
        for member in self.members.iter() {
            statements.push(dot::Statement::Edge(dot::EdgeStatement {
                left: self.name.clone().into(),
                right: dot::EdgeRHS {
                    r#type: dot::EdgeType::Directional,
                    id: member.entity.clone().into(),
                    right: Box::new(None),
                },
                attributes: Some(dot::AttributeList {
                    content: dot::AList(vec![dot::AListItem {
                        key: "arrowhead".into(),
                        value: match member.optionality {
                            RelationOptionality::Optional => "odot",
                            RelationOptionality::Required => "tee",
                        }
                        .into(),
                    }]),
                    tail: Box::new(None),
                }),
            }));
        }
        statements
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ERDError {
    DuplicateIdent(Ident),
    DuplicateAttributeInEntity(Ident, Ident), // Attribute, Entity
    DuplicateAttributeInRelation(Ident, Ident), // Attribute, Relation
    UnknownEntityInRelation(Ident, Ident),    // Entity, Relation
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
                Expr::Relation(name, label, members) => Some(Relation {
                    name: name.clone(),
                    label: label.clone(),
                    members: members.clone(),
                    attributes: Vec::new(), // TODO
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

        statements.extend(self.entities.to_dot_statements());
        statements.extend(self.relations.to_dot_statements());

        statements
    }
}

impl ERD {
    pub fn to_dot(&self) -> dot::Graph {
        dot::Graph {
            strict: false,
            r#type: dot::GraphType::Directional,
            id: None, // TODO
            statements: self.to_dot_statements(),
        }
    }
}
