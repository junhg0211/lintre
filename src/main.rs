mod ast;
mod parser;
mod interpreter;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <program.txt>", args[0]);
        std::process::exit(1);
    }

    let code = fs::read_to_string(&args[1]).expect("Failed to read program file");
    let mut parser = parser::Parser::new(&code);
    let ast = parser.parse().expect("Parse error!");

    let mut interpreter = interpreter::Interpreter::new();
    let result = interpreter.eval(ast).expect("Evaluation error!");

    println!("{:?}", result);
}