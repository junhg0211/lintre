use crate::ast::Expr;
use std::str::Chars;
use std::iter::Peekable;

pub struct Parser<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Self {
        Parser { input: code.chars().peekable() }
    }

    pub fn parse(mut self) -> Result<Expr, String> {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        let mut exprs = vec![self.parse_primary()?];
        while let Some(&ch) = self.input.peek() {
            if ch == ';' {
                self.input.next();
                exprs.push(self.parse_primary()?);
            } else {
                break;
            }
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();
        match self.input.peek() {
            Some('L') => {
                self.input.next();
                let params = self.parse_words()?;
                self.expect('.')?;
                let body = self.parse_expression()?;
                Ok(Expr::Function(params, Box::new(body)))
            }
            Some('(') => {
                self.input.next();
                let inner = self.parse_expression()?;
                self.expect(')')?;
                Ok(Expr::Paren(Box::new(inner)))
            }
            Some(ch) if ch.is_alphanumeric() || *ch == '_' => {
                let word = self.parse_word()?;
                self.skip_whitespace();
                if let Some('=') = self.input.peek() {
                    self.input.next();
                    let expr = self.parse_expression()?;
                    Ok(Expr::Define(word, Box::new(expr)))
                } else {
                    let mut words = vec![Expr::Word(word)];
                    while let Some(' ') = self.input.peek() {
                        self.input.next();
                        words.push(Expr::Word(self.parse_word()?));
                    }
                    if words.len() == 1 {
                        Ok(words.pop().unwrap())
                    } else {
                        Ok(Expr::Words(words))
                    }
                }
            }
            Some(c) => Err(format!("Unexpected character: {}", c)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_word(&mut self) -> Result<String, String> {
        let mut word = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        if word.is_empty() {
            Err("Expected word".to_string())
        } else {
            Ok(word)
        }
    }

    fn parse_words(&mut self) -> Result<Vec<String>, String> {
        let mut words = vec![self.parse_word()?];
        while let Some(' ') = self.input.peek() {
            self.input.next();
            words.push(self.parse_word()?);
        }
        Ok(words)
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        self.skip_whitespace();
        match self.input.next() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(format!("Expected '{}', got '{}'", ch, c)),
            None => Err(format!("Expected '{}', got EOF", ch)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }
}

pub fn parse(code: &str) -> Result<Expr, String> {
    Parser::new(code).parse()
}
