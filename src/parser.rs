use crate::ast::Expr;
use regex::Regex;

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    word_re: Regex,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            word_re: Regex::new(r"^[a-zA-Z0-9_]+").unwrap(),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.next_char();
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
            exprs.push(self.parse_expression()?);
            self.skip_whitespace();
            if self.peek_char() == Some(';') {
                self.next_char();
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
        match self.peek_char() {
            Some('(') => {
                self.next_char();
                let expr = self.parse_document()?;
                self.skip_whitespace();
                if self.next_char() != Some(')') {
                    return Err("Expected ')'".to_string());
                }
                Ok(expr)
            }
            Some('L') => {
                self.next_char();
                self.skip_whitespace();
                let params = self.parse_words()?;
                self.skip_whitespace();
                if self.next_char() != Some('.') {
                    return Err("Expected '.' after lambda parameters".to_string());
                }
                let body = self.parse_expression()?;
                Ok(Expr::Lambda(params, Box::new(body)))
            }
            Some(_) => {
                let word = self.parse_word()?;
                self.skip_whitespace();
                if self.peek_char() == Some('=') {
                    self.next_char();
                    let expr = self.parse_expression()?;
                    Ok(Expr::Define(word, Box::new(expr)))
                } else {
                    let mut expr = Expr::Var(word);
                    while {
                        self.skip_whitespace();
                        match self.peek_char() {
                            Some(c) if c == 'L' || c == '(' || c.is_ascii_alphanumeric() || c == '_' => true,
                            _ => false,
                        }
                    } {
                        let rhs = self.parse_expression()?;
                        expr = Expr::Apply(Box::new(expr), Box::new(rhs));
                    }
                    Ok(expr)
                }
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_words(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        loop {
            self.skip_whitespace();
            if let Some(c) = self.peek_char() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    params.push(self.parse_word()?);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if params.is_empty() {
            Err("Expected at least one word after 'L'".to_string())
        } else {
            Ok(params)
        }
    }

    fn parse_word(&mut self) -> Result<String, String> {
        let input = &self.input[self.pos..];
        if let Some(mat) = self.word_re.find(input) {
            self.pos += mat.end();
            Ok(mat.as_str().to_string())
        } else {
            Err("Expected word".to_string())
        }
    }
}
