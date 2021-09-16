use std::convert::TryInto;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ident(pub String);

impl std::convert::From<String> for Ident {
    fn from(s: String) -> Self {
        Ident(s)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Attribute {
    Normal(Ident),
    Key(Ident),
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
    cardinality: RelationCardinality,
    optionality: RelationOptionality,
    entity: Ident,
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
    /// Matches a relation with an optional name and members
    Relation(Ident, Option<String>, Vec<RelationMember>),
}
