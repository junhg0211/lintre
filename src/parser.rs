use std::iter::Peekable;
use std::str::Chars;

use crate::ast::Expr;

pub struct Parser<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            input: source.chars().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        let mut exprs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.input.peek().is_none() {
                break;
            }
            let expr = self.parse_expression()?;
            exprs.push(expr);
            self.skip_whitespace();
            if self.consume(';') {
                continue;
            } else {
                break;
            }
        }
        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();
        if self.peek_is('L') {
            self.parse_function()
        } else if self.peek_is('(') {
            self.parse_paren()
        } else if let Some(expr) = self.parse_define()? {
            Ok(expr)
        } else {
            self.parse_words()
        }
    }

    fn parse_function(&mut self) -> Result<Expr, String> {
        self.expect('L')?;
        self.skip_whitespace();
        let mut params = vec![self.parse_word()?];
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                params.push(self.parse_word()?);
            } else if ch == ' ' {
                self.input.next();
            } else {
                break;
            }
        }
        self.expect('.')?;
        let body = self.parse_expression()?;
        Ok(Expr::Function(params, Box::new(body)))
    }

    fn parse_define(&mut self) -> Result<Option<Expr>, String> {
        let saved_input = self.input.clone();
        self.skip_whitespace();
        if let Ok(name) = self.parse_word() {
            self.skip_whitespace();
            if self.consume('=') {
                self.skip_whitespace();
                let body = self.parse_primary()?; // !!! 여기 수정: parse_primary()로 딱 하나만 읽기
                return Ok(Some(Expr::Define(name, Box::new(body))));
            }
        }
        self.input = saved_input;
        Ok(None)
    }

    fn parse_words(&mut self) -> Result<Expr, String> {
        let mut words = vec![Expr::Word(self.parse_word()?)];
        self.skip_whitespace();
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                words.push(Expr::Word(self.parse_word()?));
                self.skip_whitespace();
            } else {
                break;
            }
        }
        if words.len() == 1 {
            Ok(words.into_iter().next().unwrap())
        } else {
            Ok(Expr::Words(words))
        }
    }

    fn parse_paren(&mut self) -> Result<Expr, String> {
        self.expect('(')?;

        let mut exprs = vec![self.parse_expression()?];

        while self.peek_is(';') {
            self.expect(';')?;
            exprs.push(self.parse_expression()?);
        }

        self.expect(')')?;

        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();
        if self.peek_is('L') {
            self.parse_function()
        } else if self.peek_is('(') {
            self.parse_paren()
        } else {
            self.parse_words()
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

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch == ' ' || ch == '\n' || ch == '\t' || ch == '\r' {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.input.peek() == Some(&expected) {
            self.input.next();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.input.next() == Some(expected) {
            Ok(())
        } else {
            Err(format!("Expected '{}'", expected))
        }
    }

    fn peek_is(&mut self, expected: char) -> bool {
        self.input.peek() == Some(&expected)
    }
}
