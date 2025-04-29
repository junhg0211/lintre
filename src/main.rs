mod tokenizer;
mod parser;
mod ast;
mod interpreter;

use tokenizer::tokenize;
use parser::Parser;
use ast::Expr;
use interpreter::{Env, Value, eval};

pub enum TraceMode {
    None,
    Last,
    All,
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let (trace_mode, filename) = match args.get(1) {
        Some(flag) if flag == "-b" => (TraceMode::Last, args.get(2).ok_or("No filename provided")?),
        Some(flag) if flag == "-B" => (TraceMode::All, args.get(2).ok_or("No filename provided")?),
        Some(file) => (TraceMode::None, file),
        None => return Err("No filename provided".to_string()),
    };

    let input = std::fs::read_to_string(filename.as_str())
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let tokens = tokenize(&input)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_document()?;

    let mut env = Env::new();
    let mut step_count = 0;
    let result = eval_document(&ast, &mut env, &mut step_count, &trace_mode)?;

    pretty_print_value_with_env(&result, &env);

    Ok(())
}

fn eval_document(expr: &Expr, env: &mut Env, step_count: &mut usize, trace_mode: &TraceMode) -> Result<Value, String> {
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
                last = eval(expr, env, trace, step_count)?;
            }
            Ok(last)
        }
        _ => {
            let trace = matches!(trace_mode, TraceMode::Last | TraceMode::All);
            eval(expr, env, trace, step_count)
        }
    }
}

fn pretty_print_value_with_env(value: &Value, env: &Env) {
    for (name, captured_val) in env {
        if *captured_val == *value {
            println!("{}", name);
            return;
        }
    }

    match value {
        Value::Closure(params, body, capture_env) => {
            print!("Closure(");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", param);
            }
            print!(") -> ");
            pretty_print_expr(body);
            println!();

            if !capture_env.is_empty() {
                println!("Captured:");
                for (k, v) in capture_env {
                    print!("  {}: ", k);
                    match v {
                        Value::Closure(_, _, _) => {
                            println!("Closure(...)");
                        }
                        Value::Unit => {
                            println!("unit");
                        }
                    }
                }
            }
        }
        Value::Unit => {
            println!("unit");
        }
    }
}

fn pretty_print_expr(expr: &Expr) {
    match expr {
        Expr::Var(name) => {
            print!("{}", name);
        }
        Expr::Apply(f, arg) => {
            print!("(");
            pretty_print_expr(f);
            print!(" ");
            pretty_print_expr(arg);
            print!(")");
        }
        Expr::Lambda(params, body) => {
            print!("L ");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print!("{}", param);
            }
            print!(". ");
            pretty_print_expr(body);
        }
        Expr::Define(name, expr) => {
            print!("{} = ", name);
            pretty_print_expr(expr);
        }
        Expr::Sequence(exprs) => {
            print!("(");
            for (i, e) in exprs.iter().enumerate() {
                if i > 0 {
                    print!("; ");
                }
                pretty_print_expr(e);
            }
            print!(")");
        }
    }
}
