mod tokenizer;
mod parser;
mod ast;
mod interpreter;

use tokenizer::tokenize;
use parser::Parser;
use ast::Expr;
use interpreter::{Env, Value, eval};

use std::collections::HashSet;

pub enum TraceMode {
    None,
    Last,
    All,
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let (trace_mode, filename) = match args.get(1) {
        Some(flag) if flag == "-b" => (TraceMode::Last, args.get(2).ok_or("No filename")?),
        Some(flag) if flag == "-B" => (TraceMode::All, args.get(2).ok_or("No filename")?),
        Some(file) => (TraceMode::None, file),
        None => return Err("No filename provided".to_string()),
    };

    let input = std::fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    println!("--- Source ---\n{}\n", input);

    let tokens = tokenize(&input)?;
    println!("--- Tokens ---");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_document()?;

    println!("--- AST ---");
    pretty_print_ast(&ast, 0);
    println!();

    println!("--- Evaluation ---");

    let mut env = Env::new();
    let mut seen: HashSet<Expr> = HashSet::new();
    let result = eval_document(&ast, &mut env, &mut seen, &trace_mode)?;

    println!("\n--- Result ---");
    println!("{:?}", result);

    Ok(())
}

pub fn eval_document(
    expr: &Expr,
    env: &mut Env,
    seen: &mut HashSet<Expr>,
    trace_mode: &TraceMode,
) -> Result<Value, String> {
    match expr {
        Expr::Sequence(exprs) => {
            let mut last = Value::Unit;
            for (i, expr) in exprs.iter().enumerate() {
                let is_last = i == exprs.len() - 1;
                let trace = match trace_mode {
                    TraceMode::None => false,
                    TraceMode::Last => is_last,
                    TraceMode::All => true,
                };
                last = eval(expr, env, trace, seen)?;
            }
            Ok(last)
        }
        _ => {
            let trace = matches!(trace_mode, TraceMode::Last | TraceMode::All);
            eval(expr, env, trace, seen)
        }
    }
}

fn pretty_print_ast(expr: &Expr, indent: usize) {
    let pad = " ".repeat(indent);
    match expr {
        Expr::Var(name) => println!("{}Var({})", pad, name),
        Expr::Lambda(params, body) => {
            println!("{}Lambda({:?})", pad, params);
            pretty_print_ast(body, indent + 2);
        }
        Expr::Apply(f, arg) => {
            println!("{}Apply(", pad);
            pretty_print_ast(f, indent + 2);
            pretty_print_ast(arg, indent + 2);
            println!("{})", pad);
        }
        Expr::Define(name, expr) => {
            println!("{}Define({})", pad, name);
            pretty_print_ast(expr, indent + 2);
        }
        Expr::Sequence(exprs) => {
            println!("{}Sequence(", pad);
            for e in exprs {
                pretty_print_ast(e, indent + 2);
            }
            println!("{})", pad);
        }
    }
}
