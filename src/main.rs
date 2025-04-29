use std::env;
use std::fs;

mod ast;
mod parser;
mod interpreter;

use interpreter::{Env, eval, normalize};
use parser::Parser;
use ast::Expr;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).ok_or("No filename provided")?;

    let input = fs::read_to_string(filename).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(&input);
    let ast = parser.parse()?;

    let mut env = Env::new();
    let mut step_count = 0;
    let result = eval(&ast, &mut env, false, &mut step_count)?;

    println!("{}", pretty_print_value(&result));
    Ok(())
}

fn pretty_print_value(value: &interpreter::Value) -> String {
    match value {
        interpreter::Value::Closure(params, body, _) => {
            let param_str = params.join(" ");
            format!("L {}. {}", param_str, pretty_print_expr(body))
        }
        interpreter::Value::Unit => String::from("Unit"),
    }
}

fn pretty_print_expr(expr: &Expr) -> String {
    match expr {
        Expr::Var(name) => name.clone(),
        Expr::Lambda(params, body) => {
            let param_str = params.join(" ");
            format!("L {}. {}", param_str, pretty_print_expr(body))
        }
        Expr::Apply(f, arg) => {
            format!("({} {})", pretty_print_expr(f), pretty_print_expr(arg))
        }
        Expr::Define(name, e) => {
            format!("{} = {}", name, pretty_print_expr(e))
        }
        Expr::Sequence(exprs) => {
            exprs.iter().map(pretty_print_expr).collect::<Vec<_>>().join("; ")
        }
    }
}
