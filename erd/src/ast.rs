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
pub enum AttributeType {
    Normal,
    Key,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub ident: Ident,
    pub r#type: AttributeType,
    pub datatype: Option<DataType>,
}

impl Attribute {
    pub fn get_ident(&self) -> Ident {
        self.ident.to_owned()
    }
    pub fn get_type(&self) -> AttributeType {
        self.r#type.to_owned()
    }
    pub fn get_data_type(&self) -> Option<DataType> {
        self.datatype.to_owned()
    }
    pub fn renamed(&self, new_name: Ident) -> Self {
        Self {
            ident: new_name,
            r#type: self.r#type.clone(),
            datatype: self.datatype.clone(),
        }
    }
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_ident().hash(state)
    }
}

impl std::convert::From<(String, String, Option<String>)> for Attribute {
    fn from((r#type, name, datatype): (String, String, Option<String>)) -> Self {
        let ident = name.into();
        Self {
            ident,
            r#type: match &r#type[..] {
                "id" => AttributeType::Key,
                "attribute" => AttributeType::Normal,
                _ => unreachable!(),
            },
            datatype: datatype.map(|d| d.into()),
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
pub struct ForeignKey {
    pub attribute_names: Vec<Ident>,
    pub relation: Ident,
}

impl std::convert::From<(Vec<String>, String)> for ForeignKey {
    fn from((attrs, relation): (Vec<String>, String)) -> Self {
        Self {
            attribute_names: attrs.into_iter().map(|a| a.into()).collect(),
            relation: relation.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// Matches an entity with attributes
    Entity(Ident, Vec<Attribute>),
    /// Matches a relation with an optional name, members and attributes
    Relation(Ident, Option<String>, Vec<RelationMember>, Vec<Attribute>),
    /// Matches a table with a name based on an entity with some foreign key settings
    EntityTable(Ident, Ident, Vec<ForeignKey>),
    /// Matches a table with a name based on a relation
    RelationTable(Ident, Ident),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    Integer,
    AutoIncrement,
    Float,
    Boolean,
    Date,
    Time,
    DateTime,
    Varchar(usize),
}

impl std::convert::From<String> for DataType {
    fn from(s: String) -> Self {
        if s.starts_with("varchar") {
            Self::Varchar(s["varchar(".len()..(s.len() - 1)].parse().unwrap())
        } else {
            match &s[..] {
                "integer" => Self::Integer,
                "autoincrement" => Self::AutoIncrement,
                "float" => Self::Float,
                "boolean" => Self::Boolean,
                "date" => Self::Date,
                "time" => Self::Time,
                "datetime" => Self::DateTime,
                _ => unreachable!(),
            }
        }
    }
}

impl DataType {
    pub fn foreign_key_type(&self) -> DataType {
        match self {
            Self::Integer => Self::Integer,
            Self::AutoIncrement => Self::Integer,
            Self::Float => Self::Float,
            Self::Boolean => Self::Boolean,
            Self::Date => Self::Date,
            Self::Time => Self::Time,
            Self::DateTime => Self::DateTime,
            Self::Varchar(n) => Self::Varchar(*n),
        }
    }
}
