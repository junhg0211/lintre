use std::env;
use std::fs;

mod ast;
mod parser;
mod interpreter;

use ast::Expr;
use interpreter::{Env, eval, Value};
use parser::Parser;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).ok_or("No filename provided")?;

    let input = fs::read_to_string(filename).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(&input);
    let ast = parser.parse()?;

    let mut env = Env::new();
    let result = eval(&ast, &mut env)?;

    if let Some(name) = env.iter().find_map(|(k, v)| if *v == result { Some(k) } else { None }) {
        println!("{}", name);
    } else {
        println!("{}", pretty_print_value(&result));
    }

    Ok(())
}

fn pretty_print_value(val: &Value) -> String {
    match val {
        Value::Closure(params, body, _) => {
            format!("L {}. {}", params.join(" "), pretty_print_expr(body))
        }
        Value::Unit => "Unit".to_string(),
    }
}

fn pretty_print_expr(expr: &Expr) -> String {
    match expr {
        Expr::Var(name) => name.clone(),
        Expr::Lambda(params, body) => format!("L {}. {}", params.join(" "), pretty_print_expr(body)),
        Expr::Apply(f, arg) => format!("({} {})", pretty_print_expr(f), pretty_print_expr(arg)),
        Expr::Define(name, rhs) => format!("{} = {}", name, pretty_print_expr(rhs)),
        Expr::Sequence(exprs) => exprs.iter().map(pretty_print_expr).collect::<Vec<_>>().join("; "),
    }
}