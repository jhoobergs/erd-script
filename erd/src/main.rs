mod ast;
mod parser;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use parser::ConsumeError;

fn parse_file(path: &std::path::Path) -> Result<(), ConsumeError> {
    println!("{:?}", path.display());
    let content = std::fs::read_to_string(path).expect("Valid file");
    let pairs =
        parser::parse_as_erd(&content).map_err(|e| parser::ConsumeError::ERDParseError(vec![e]))?;
    println!("{:?}", pairs);
    let asts = parser::consume_expressions(pairs)?;
    println!("{:?}\n", asts);
    Ok(())
}

fn main() {
    println!("todo")
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_examples() -> Result<(), ConsumeError> {
        let paths = std::fs::read_dir("../examples").unwrap();

        for path in paths {
            parse_file(&path.unwrap().path())?;
        }

        Ok(())
    }
}
