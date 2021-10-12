use crate::ast;
use pest::error::Error;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::{Parser, Span};

#[derive(Parser)]
#[grammar = "erd.pest"]
pub struct ERDParser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserNode<'i> {
    pub expr: ParserExpr,
    pub span: Span<'i>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParserExpr {
    /// (Name, Vec<("id" | "attribute", Name, Option<type>)> )
    Entity(String, Vec<(String, String, Option<String>)>),
    /// (Name, Optional label, Vec<RelationMembers>, Vec<("id" | "attribute", Name, Option<type>)>)
    Relation(
        String,
        Option<String>,
        Vec<(String, String, String)>,
        Vec<(String, String, Option<String>)>,
    ),
    /// (name, entity, Vec<fk_name, fk_rel>)
    EntityTable(String, String, Vec<(String, String)>),
    /// (name, relation
    RelationTable(String, String),
}

impl<'i> std::convert::From<ParserNode<'i>> for ast::Expr {
    fn from(node: ParserNode<'i>) -> ast::Expr {
        node.expr.into()
    }
}
impl<'i> std::convert::From<ParserExpr> for ast::Expr {
    fn from(expr: ParserExpr) -> ast::Expr {
        match expr {
            ParserExpr::Entity(name, attributes) => ast::Expr::Entity(
                name.into(),
                attributes.into_iter().map(|a| a.into()).collect(),
            ),
            ParserExpr::Relation(name, label_option, members, attributes) => ast::Expr::Relation(
                name.into(),
                label_option.map(|l| l.into()),
                members.into_iter().map(|m| m.into()).collect(),
                attributes.into_iter().map(|m| m.into()).collect(),
            ),
            ParserExpr::EntityTable(name, er, foreign_keys) => ast::Expr::EntityTable(
                name.into(),
                er.into(),
                foreign_keys.into_iter().map(|f| f.into()).collect(),
            ),
            ParserExpr::RelationTable(name, er) => ast::Expr::RelationTable(name.into(), er.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsumeError {
    ERDParseError(Vec<Error<Rule>>),
    UnknownParseError,
}

impl std::fmt::Display for ConsumeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConsumeError::ERDParseError(errors) => {
                write!(
                    f,
                    "Error while parsing ERD:\n{}",
                    errors
                        .iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            ConsumeError::UnknownParseError => write!(f, "Unknown parse error, please report."),
        }
    }
}

fn consume_expression_with_spans(pairs: Pairs<Rule>) -> Result<Vec<ParserNode>, Vec<Error<Rule>>> {
    let mut results = Vec::new();
    for expression in pairs.filter(|pair| pair.as_rule() == Rule::expression) {
        results.push(consume_expression(expression)?);
    }
    Ok(results)
}

pub fn consume_expressions(pairs: Pairs<Rule>) -> Result<Vec<ast::Expr>, ConsumeError> {
    let res_res = std::panic::catch_unwind(|| {
        let expression = consume_expression_with_spans(pairs)?;
        //let errors = validator::validate_ast(&rules);
        //if errors.is_empty() {
        Ok(expression.into_iter().map(|e| e.into()).collect())
        /*} else {
            Err(errors)
        }*/
    });
    match res_res {
        Ok(res) => res.map_err(ConsumeError::ERDParseError),
        Err(_err) => Err(ConsumeError::UnknownParseError),
    }
}

fn consume_expression(expression: Pair<Rule>) -> Result<ParserNode, Vec<Error<Rule>>> {
    let pair = expression.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::entity => {
            let mut pairs = pair.into_inner();
            let pair = pairs.next().unwrap();
            let name = pair.as_str().trim().to_string();
            let mut attributes = Vec::new();
            for attribute in pairs {
                let mut pairs = attribute.into_inner();
                attributes.push((
                    pairs.next().unwrap().as_str().to_string(),
                    pairs.next().unwrap().as_str().to_string(),
                    pairs.next().map(|pair| pair.as_str().to_string()),
                ))
            }

            Ok(ParserNode {
                expr: ParserExpr::Entity(name, attributes),
                span: pair.as_span(),
            })
        }
        Rule::relation => {
            let mut pairs = pair.into_inner().peekable();
            let pair = pairs.next().unwrap();
            let name = pair.as_str().trim().to_string();
            let mut members = Vec::new();
            let mut attributes = Vec::new();
            let mut label: Option<String> = None;
            let peek = pairs.peek();
            if let Some(pair) = peek {
                if pair.as_rule() == Rule::relation_name {
                    let pair = pairs.next().unwrap();
                    label = Some(pair.as_str().to_string());
                }
            }
            for item in pairs {
                match item.as_rule() {
                    Rule::member => {
                        let mut pairs = item.into_inner();
                        members.push((
                            pairs.next().unwrap().as_str().to_string(),
                            pairs.next().unwrap().as_str().to_string(),
                            pairs.next().unwrap().as_str().to_string(),
                        ))
                    }
                    Rule::attribute => {
                        let mut pairs = item.into_inner();
                        attributes.push((
                            pairs.next().unwrap().as_str().to_string(),
                            pairs.next().unwrap().as_str().to_string(),
                            pairs.next().map(|pair| pair.as_str().to_string()),
                        ))
                    }
                    _ => unreachable!(),
                }
            }

            Ok(ParserNode {
                expr: ParserExpr::Relation(name, label, members, attributes),
                span: pair.as_span(),
            })
        }
        Rule::entity_table => {
            let mut foreign_keys = Vec::new();
            let mut pairs = pair.into_inner().peekable();
            let pair = pairs.next().unwrap();
            let name = pair.as_str().to_string();
            let pair = pairs.next().unwrap();
            let entity = pair.as_str().to_string();

            for pair in pairs {
                match pair.as_rule() {
                    Rule::foreign => {
                        let mut pairs = pair.into_inner();
                        foreign_keys.push((
                            pairs.next().unwrap().as_str().to_string(),
                            pairs.next().unwrap().as_str().to_string(),
                        ));
                    }
                    _ => unreachable!(),
                }
            }

            Ok(ParserNode {
                expr: ParserExpr::EntityTable(name, entity, foreign_keys),
                span: pair.as_span(),
            })
        }
        Rule::relation_table => {
            let mut pairs = pair.into_inner().peekable();
            let pair = pairs.next().unwrap();
            let name = pair.as_str().to_string();
            let pair = pairs.next().unwrap();
            let relation = pair.as_str().to_string();

            Ok(ParserNode {
                expr: ParserExpr::RelationTable(name, relation),
                span: pair.as_span(),
            })
        }
        _ => unreachable!(),
    }
}

pub fn parse_as_erd(s: &str) -> Result<Pairs<'_, Rule>, pest::error::Error<Rule>> {
    ERDParser::parse(Rule::erd, s)
}
