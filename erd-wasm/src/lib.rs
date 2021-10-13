use erd_script::sql::SQL;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn compile_dot(s: &str);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ParsingError {
    Known(Vec<String>),
    Unknown,
}

impl ParsingError {
    fn create(item: erd_script::parser::ConsumeError) -> Self {
        match item {
            erd_script::parser::ConsumeError::UnknownParseError => Self::Unknown,
            erd_script::parser::ConsumeError::ERDParseError(v) => {
                Self::Known(v.into_iter().map(|e| format!("{:?}", e)).collect())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ERDCompileError {
    ERDErrors(Vec<String>),
    ParsingError(ParsingError),
}

impl ERDCompileError {
    fn create(item: erd_script::erd::ERDFromScriptError) -> Self {
        match item {
            erd_script::erd::ERDFromScriptError::ERDError(v) => {
                Self::ERDErrors(v.into_iter().map(|a| format!("{}", a)).collect())
            }
            erd_script::erd::ERDFromScriptError::ParsingError(p) => {
                Self::ParsingError(ParsingError::create(p))
            }
        }
    }
}

#[wasm_bindgen]
pub fn compile_erd(erd_script: &str) -> JsValue {
    JsValue::from_serde(
        &erd_script::erd::ERD::from_script(erd_script)
            .map(|erd| erd.to_dot().to_string())
            .map_err(ERDCompileError::create),
    )
    .unwrap_or(false.into())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum PhysicalCompileError {
    ERDErrors(Vec<String>),
    PhysicalErrors(Vec<String>),
    ParsingError(ParsingError),
    InvalidDBMS,
}

impl PhysicalCompileError {
    fn create(item: erd_script::physical::PhysicalFromScriptError) -> Self {
        match item {
            erd_script::physical::PhysicalFromScriptError::ERDError(v) => {
                Self::ERDErrors(v.into_iter().map(|a| format!("{}", a)).collect())
            }
            erd_script::physical::PhysicalFromScriptError::PhysicalError(v) => {
                Self::PhysicalErrors(v.into_iter().map(|a| format!("{}", a)).collect())
            }
            erd_script::physical::PhysicalFromScriptError::ParsingError(p) => {
                Self::ParsingError(ParsingError::create(p))
            }
        }
    }
}

#[wasm_bindgen]
pub fn compile_physical(erd_script: &str, sql_dbms: &str) -> JsValue {
    if let Some(dbms) = SQL::from_str(sql_dbms) {
        JsValue::from_serde(
            &erd_script::physical::PhysicalDescription::from_script(erd_script)
                .map(|physical| {
                    let mut s = String::new();
                    physical.to_physical().write_sql_create(&mut s, dbms);
                    s
                })
                .map_err(PhysicalCompileError::create),
        )
        .unwrap_or(false.into())
    } else {
        JsValue::from_serde(&PhysicalCompileError::InvalidDBMS).unwrap_or(false.into())
    }
}
