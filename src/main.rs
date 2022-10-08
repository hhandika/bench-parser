mod cli;
mod parser;
mod types;

use parser::Parser;

fn main() {
    let matches = cli::parser_arg();
    let input_files = cli::parse_input(&matches);
    let output = cli::parse_output(&matches);
    Parser::new(&input_files, output)
        .parse_benchmark()
        .expect("Failed parsing benchmark");
}
