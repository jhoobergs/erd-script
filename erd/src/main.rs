mod ast;
mod dot;
mod erd;
mod parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use parser::ConsumeError;

fn parse_file(path: &std::path::Path) -> Result<Vec<ast::Expr>, ConsumeError> {
    println!("{:?}", path.display());
    let content = std::fs::read_to_string(path).expect("Valid file");
    let pairs =
        parser::parse_as_erd(&content).map_err(|e| parser::ConsumeError::ERDParseError(vec![e]))?;
    println!("{:?}", pairs);
    let asts = parser::consume_expressions(pairs)?;
    println!("{:?}\n", asts);
    Ok(asts)
}

fn main() {
    println!("todo")
}

#[cfg(test)]
mod test {
    use super::*;
    use erd::ERD;
    use std::convert::TryInto;
    #[test]
    fn compile_examples() -> Result<(), ConsumeError> {
        let paths = std::fs::read_dir("../examples").unwrap();

        for path in paths.filter(|p| {
            p.as_ref().unwrap().path().extension() == Some(&std::ffi::OsStr::new("erd"))
        }) {
            let path = path.unwrap().path();
            let expr = parse_file(&path)?;
            let erd: Result<ERD, _> = expr.try_into();
            let dot = erd.unwrap().to_dot().to_string();
            println!("{}", dot);
            std::fs::write("../examples/tmp.dot", dot).expect("failed writing");
            let output = std::process::Command::new("dot")
                .arg("-Tsvg")
                .arg("../examples/tmp.dot")
                .output()
                .expect("failed converting with dot");
            let new_path = path.with_extension("svg");
            std::fs::write(new_path, output.stdout).expect("failed writing svg");
        }

        Ok(())
    }
}
