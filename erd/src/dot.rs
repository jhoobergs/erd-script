// See https://graphviz.org/doc/info/lang.html
use std::fmt;

type ID = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    strict: bool,
    r#type: GraphType,
    id: Option<ID>,
    statements: Vec<Statement>,
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.strict {
            write!(f, "strict ")?;
        }
        write!(f, "{} ", self.r#type)?;
        if let Some(id) = &self.id {
            write!(f, "{} ", id)?;
        }
        write!(f, "{{\n")?;
        for statement in &self.statements {
            writeln!(f, "{}", statement)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphType {
    Normal,
    Directional,
}

impl fmt::Display for GraphType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "graph"),
            Self::Directional => write!(f, "digraph"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Node(NodeStatement),
    Edge(EdgeStatement),
    Attribute(AttributeStatement),
    ID,       // TODO
    Subgraph, // TODO
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(s) => write!(f, "{}", s),
            Self::Edge(s) => write!(f, "{}", s),
            Self::Attribute(s) => write!(f, "{}", s),
            _ => Ok(()), // TODO
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AListItem {
    key: ID,
    value: ID,
}

impl fmt::Display for AListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AList(Vec<AListItem>);

impl fmt::Display for AList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.iter();
        let first = it.next();
        if let Some(first) = first {
            write!(f, "{}", first)?;
            for item in it {
                write!(f, "; {}", item)?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeList {
    content: AList,
    tail: Box<Option<AttributeList>>,
}

impl fmt::Display for AttributeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref tail) = *self.tail {
            write!(f, "[{}] {}", self.content, tail)
        } else {
            write!(f, "{}", self.content)
        }
    }
}

type NodeId = ID; // TODO

#[derive(Debug, Clone, PartialEq)]
pub struct NodeStatement {
    node: NodeId,
    attributes: Option<AttributeList>,
}

impl fmt::Display for NodeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref attributes) = self.attributes {
            write!(f, "{} {}", self.node, attributes)
        } else {
            write!(f, "{}", self.node)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeStatement {
    left: NodeId, // TODO: subgraph
    right: EdgeRHS,
    attributes: Option<AttributeList>,
}

impl fmt::Display for EdgeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref attributes) = self.attributes {
            write!(f, "{} {} {}", self.left, self.right, attributes)
        } else {
            write!(f, "{} {}", self.left, self.right)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Normal,
    Directional,
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "--"),
            Self::Directional => write!(f, "->"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRHS {
    r#type: EdgeType, // TODO: validate this
    id: NodeId,       // TODO: subgraph
    right: Box<Option<EdgeRHS>>,
}

impl fmt::Display for EdgeRHS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref right) = *self.right {
            write!(f, "{} {} {}", self.r#type, self.id, right)
        } else {
            write!(f, "{} {}", self.r#type, self.id)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeStatement {
    r#type: AttributeStatementType,
    attributes: AttributeList,
}

impl fmt::Display for AttributeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.r#type, self.attributes)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeStatementType {
    Graph,
    Node,
    Edge,
}

impl fmt::Display for AttributeStatementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph => write!(f, "graph"),
            Self::Node => write!(f, "node"),
            Self::Edge => write!(f, "edge"),
        }
    }
}
