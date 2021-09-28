use erd_script::erd::ToDot;
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
enum CompileError {
    ERDErrors(Vec<String>),
    ParsingError(ParsingError),
}

impl CompileError {
    fn create(item: erd_script::erd::FromScriptError) -> Self {
        match item {
            erd_script::erd::FromScriptError::ERDError(v) => {
                Self::ERDErrors(v.into_iter().map(|a| format!("{}", a)).collect())
            }
            erd_script::erd::FromScriptError::ParsingError(p) => {
                Self::ParsingError(ParsingError::create(p))
            }
        }
    }
}

#[wasm_bindgen]
pub fn compile(erd_script: &str) -> JsValue {
    JsValue::from_serde(
        &erd_script::erd::ERD::from_script(erd_script)
            .map(|erd| erd.to_dot().to_string())
            .map_err(CompileError::create),
    )
    .unwrap_or(false.into())
}
