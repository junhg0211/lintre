use std::env;

mod ast;
mod parser;
mod interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    let filename: &str;

    if args.len() == 2 {
        filename = &args[1];
    } else if args.len() == 3 && args[1] == "-b" {
        debug = true;
        filename = &args[2];
    } else {
        eprintln!("Usage: {} [-b] <source-file>", args[0]);
        std::process::exit(1);
    }

    let code = std::fs::read_to_string(filename)
        .expect("Failed to read source file.");

    let mut p = parser::Parser::new(&code);
    let ast = p.parse().expect("Parse error!");

    let mut interpreter = interpreter::Interpreter::new(debug);

    match interpreter.eval(ast) {
        Ok(result) => {
            println!("{}", interpreter.format_result(&result));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
