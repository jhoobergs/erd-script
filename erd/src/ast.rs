use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ident(pub String);

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<String> for Ident {
    fn from(s: String) -> Self {
        Ident(s)
    }
}

impl std::convert::From<Ident> for String {
    fn from(i: Ident) -> Self {
        i.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Attribute {
    Normal(Ident),
    Key(Ident),
}

impl Attribute {
    pub fn get_ident(&self) -> Ident {
        match self {
            Self::Normal(i) => i.to_owned(),
            Self::Key(i) => i.to_owned(),
        }
    }
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_ident().hash(state)
    }
}

impl std::convert::From<(String, String)> for Attribute {
    fn from((r#type, name): (String, String)) -> Self {
        let ident = name.into();
        match &r#type[..] {
            "id" => Self::Key(ident),
            "attribute" => Self::Normal(ident),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationCardinality {
    One,
    Multiple,
    Exact(usize),
}

impl RelationCardinality {
    pub fn get_amount(&self, multiple_amount: char) -> (String, char) {
        match self {
            Self::One => ("1".to_string(), multiple_amount),
            Self::Multiple => (
                multiple_amount.to_string(),
                (multiple_amount as u8 - 1) as char,
            ),
            Self::Exact(n) => (n.to_string(), multiple_amount),
        }
    }
}

impl std::convert::From<String> for RelationCardinality {
    fn from(s: String) -> Self {
        if s.starts_with("exactly") {
            Self::Exact(s["exactly".len()..(s.len() - 1)].parse().unwrap())
        } else {
            match &s[..] {
                "one" => Self::One,
                "multiple" => Self::Multiple,
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationOptionality {
    Optional,
    Required,
}

impl std::convert::From<String> for RelationOptionality {
    fn from(s: String) -> Self {
        match &s[..] {
            "optional" => Self::Optional,
            "required" => Self::Required,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationMember {
    pub cardinality: RelationCardinality,
    pub optionality: RelationOptionality,
    pub entity: Ident,
}

impl std::convert::From<(String, String, String)> for RelationMember {
    fn from((cardinality, optionality, entity): (String, String, String)) -> Self {
        Self {
            cardinality: cardinality.into(),
            optionality: optionality.into(),
            entity: entity.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// Matches an entity with attributes
    Entity(Ident, Vec<Attribute>),
    /// Matches a relation with an optional name, members and attributes
    Relation(Ident, Option<String>, Vec<RelationMember>, Vec<Attribute>),
}
