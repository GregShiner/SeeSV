mod ast;
mod external;
mod parser;
mod planner;
mod tests;

use planner::ExecutionPlan;

use crate::parser::grammar;
use std::{
    collections::HashSet,
    io::{self, Read},
};

fn main() {
    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // Parse the input
    // Assuming your parser module is named 'parser' and has a parser called 'ProgramParser'
    let parser = grammar::QueryParser::new();

    let cst = parser.parse(&input);
    match cst {
        Ok(ref cst) => {
            println!(
                "Parse Tree for {}",
                &input.strip_suffix("\n").unwrap_or(&input)
            );
            println!("{cst:#?}"); // Pretty-print the debug output
        }
        Err(e) => {
            eprintln!("Parse error: {e:?}");
            std::process::exit(1);
        }
    };
    let cst = cst.unwrap();
    let plan: ExecutionPlan = cst.into();
    println!("Execution Plan:");
    println!("{plan:#?}");
    println!("Operations executed in this order:");
    let mut completed = HashSet::new();
    loop {
        let ready_ops = plan.get_ready_nodes(&completed);
        for op_id in &ready_ops {
            println!("{}: {:?}", op_id, plan.nodes.get(op_id));
            completed.insert(*op_id);
        }
        if ready_ops.is_empty() {
            break;
        }
    }
}
