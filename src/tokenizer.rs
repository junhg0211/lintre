#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Lambda,
    Dot,
    Equal,
    Semicolon,
    LParen,
    RParen,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => { chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '=' => { tokens.push(Token::Equal); chars.next(); }
            '.' => { tokens.push(Token::Dot); chars.next(); }
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else { break; }
                }
                if ident == "L" {
                    tokens.push(Token::Lambda);
                } else {
                    tokens.push(Token::Word(ident));
                }
            }
            _ => return Err(format!("Unexpected character: {}", ch)),
        }
    }

    Ok(tokens)
}
