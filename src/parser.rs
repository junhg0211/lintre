use std::iter::Peekable;
use std::str::Chars;

use crate::ast::Expr;

pub struct Parser<'a> {
    input: Peekable<Chars<'a>>,
}

pub struct Parser<'a> {
        tokens: Vec<&'a str>,
            position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokens = source
            .split_whitespace()
            .flat_map(|token| token.split_inclusive(&['(', ')', ';', '=', '.'][..]))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .collect();
        Parser { tokens, position: 0 }
    }

    fn peek(&self) -> Option<&&'a str> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) -> Option<&'a str> {
        let tok = self.tokens.get(self.position).copied();
        self.position += 1;
        tok
    }

    pub fn parse_expression_list(&mut self) -> Result<Expr, String> {
        let mut exprs = vec![self.parse_simple_expr()?];

        while let Some(&";") = self.peek() {
            self.next(); // consume ";"
            exprs.push(self.parse_simple_expr()?);
        }

        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    fn parse_simple_expr(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(&"(") => {
                self.next(); // consume "("
                let expr = self.parse_expression_list()?;
                match self.next() {
                    Some(")") => Ok(expr),
                    _ => Err("Expected ')'".to_string()),
                }
            }
            Some(&"L") => {
                self.next();
                let mut params = vec![];
                while let Some(&tok) = self.peek() {
                    if tok == "." {
                        self.next();
                        break;
                    }
                    params.push(tok.to_string());
                    self.next();
                }
                let body = self.parse_simple_expr()?;
                Ok(Expr::Function(params, Box::new(body)))
            }
            Some(&tok) if tok.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                self.next();
                if let Some(&"=") = self.peek() {
                    self.next();
                    let expr = self.parse_simple_expr()?;
                    Ok(Expr::Define(tok.to_string(), Box::new(expr)))
                } else {
                    Ok(Expr::Word(tok.to_string()))
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expression_list()
    }
}
