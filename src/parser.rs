use crate::ast::Expr;

pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == b' ' || ch == b'\n' || ch == b'\r' || ch == b'\t' {
                self.next();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_document()
    }

    fn parse_document(&mut self) -> Result<Expr, String> {
        let mut exprs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                break;
            }
            let expr = self.parse_expression()?;
            exprs.push(expr);
            self.skip_whitespace();
            if let Some(b';') = self.peek() {
                self.next();
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

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'(') => {
                self.next();
                let expr = self.parse_document()?;
                self.skip_whitespace();
                if self.next() != Some(b')') {
                    return Err("Expected ')'".to_string());
                }
                Ok(expr)
            }
            Some(b'L') => {
                self.next();
                self.skip_whitespace();
                let params = self.parse_words()?;
                self.skip_whitespace();
                if self.next() != Some(b'.') {
                    return Err("Expected '.' after lambda params".to_string());
                }
                let body = self.parse_expression()?;
                Ok(Expr::Lambda(params, Box::new(body)))
            }
            Some(ch) if is_word_char(ch) => {
                let first = self.parse_word()?;
                self.skip_whitespace();
                if let Some(b'=') = self.peek() {
                    self.next();
                    let expr = self.parse_expression()?;
                    Ok(Expr::Define(first, Box::new(expr)))
                } else {
                    let mut expr = Expr::Var(first);
                    loop {
                        self.skip_whitespace();
                        if let Some(ch) = self.peek() {
                            if is_word_char(ch) || ch == b'L' || ch == b'(' {
                                let right = self.parse_expression()?;
                                expr = Expr::Apply(Box::new(expr), Box::new(right));
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    Ok(expr)
                }
            }
            Some(_) => Err("Unexpected character".to_string()),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_words(&mut self) -> Result<Vec<String>, String> {
        let mut words = Vec::new();
        loop {
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if is_word_char(ch) {
                    words.push(self.parse_word()?);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if words.is_empty() {
            Err("Expected at least one word".to_string())
        } else {
            Ok(words)
        }
    }

    fn parse_word(&mut self) -> Result<String, String> {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if is_word_char(ch) {
                s.push(ch as char);
                self.next();
            } else {
                break;
            }
        }
        if s.is_empty() {
            Err("Expected word".to_string())
        } else {
            Ok(s)
        }
    }
}

fn is_word_char(ch: u8) -> bool {
    (ch as char).is_ascii_alphanumeric() || ch == b'_'
}