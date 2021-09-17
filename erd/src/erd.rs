use crate::ast::{Attribute, Expr, Ident, RelationMember};
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
                        e.name.clone(),
                        attribute.get_ident(),
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
        }

        errors
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    name: Ident,
    attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    name: Ident,
    label: Option<String>,
    members: Vec<RelationMember>,
}

pub enum ERDError {
    DuplicateIdent(Ident),
    DuplicateAttributeInEntity(Ident, Ident), // Attribute, Entity
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
