mod tokenizer;

use tokenizer::{tokenize, Token};

fn main() -> Result<(), String> {
    let input = "L x y. (x y)";
    println!("Input: {}", input);

    match tokenize(input) {
        Ok(tokens) => {
            println!("Tokens:");
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => {
            println!("Tokenizer error: {}", e);
        }
    }

    Ok(())
}
