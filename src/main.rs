mod tokenizer;
mod parser;
mod ast;

use tokenizer::tokenize;
use parser::Parser;
use ast::Expr;

fn main() -> Result<(), String> {
    // 1. 파일 읽기
    let input = std::fs::read_to_string("program.txt")
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    println!("--- Source ---\n{}\n", input);

    // 2. 토크나이즈
    let tokens = tokenize(&input)?;
    println!("--- Tokens ---");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();

    // 3. 파싱
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_document()?;

    // 4. AST 출력
    println!("--- AST ---");
    pretty_print_ast(&ast, 0);

    Ok(())
}

// AST를 보기 좋게 출력하는 도우미 함수
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
