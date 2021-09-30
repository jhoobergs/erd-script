use clap::{AppSettings, Clap};
use erd_script::parser::ConsumeError;
use std::convert::TryInto;

/// Compile an erd-script file to an svg
#[derive(Clap)]
#[clap(version = "1.0", author = "Jesse Hoobergs")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// The path to an erd-script file-Some input. Because this isn't an Option<T> it's required to be used
    file_path: String,
    /// The path where the output svg should be written
    output_path: String,
}

fn parse_file(path: &std::path::Path) -> Result<Vec<erd_script::ast::Expr>, ConsumeError> {
    let content = std::fs::read_to_string(path).expect("Valid file");
    let pairs = erd_script::parser::parse_as_erd(&content)
        .map_err(|e| erd_script::parser::ConsumeError::ERDParseError(vec![e]))?;
    let asts = erd_script::parser::consume_expressions(pairs)?;
    Ok(asts)
}

fn compile_dot(dot: &erd_script::dot::Graph) -> std::io::Result<std::process::Output> {
    let dot = dot.to_string();
    let dot_file = "tmp.dot";
    std::fs::write(dot_file, dot).expect("failed writing");
    std::process::Command::new("dot")
        .arg("-Tsvg")
        .arg(dot_file)
        .output()
}

fn main() {
    let opts: Opts = Opts::parse();
    let ast = parse_file(&std::path::Path::new(&opts.file_path)).expect("Failed parsing file");
    let erd: erd_script::erd::ERD = ast.try_into().expect("Error");
    let output = compile_dot(&erd.to_dot()).expect("failed converting with dot");
    std::fs::write(opts.output_path, output.stdout).expect("failed writing svg");
}

#[cfg(test)]
mod test {
    use super::*;
    use erd_script::erd::ERD;
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
            let dot = erd.unwrap().to_dot();
            let output = compile_dot(&dot).expect("failed converting with dot");
            let new_path = path.with_extension("svg");
            std::fs::write(new_path, output.stdout).expect("failed writing svg");
        }

        Ok(())
    }
}
