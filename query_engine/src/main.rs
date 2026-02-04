mod ast;
mod external;
mod parser;
mod tests;

use crate::parser::grammar;
use std::io::{self, Read};

fn main() {
    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // Parse the input
    // Assuming your parser module is named 'parser' and has a parser called 'ProgramParser'
    let parser = grammar::QueryParser::new();

    match parser.parse(&input) {
        Ok(cst) => {
            println!("{:#?}", cst); // Pretty-print the debug output
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    }
}
