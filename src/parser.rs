use crate::ast::Expr;
use regex::Regex;

pub struct Parser<'a> {
    tokens: Vec<&'a str>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let re = Regex::new(r"[\w_]+|[();=.]").unwrap();
        let tokens = re.find_iter(source)
            .map(|mat| mat.as_str())
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

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(&"(") => {
                self.next();
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
                let body = self.parse_application()?;
                Ok(Expr::Function(params, Box::new(body)))
            }
            Some(&tok) if tok.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                self.next();
                if let Some(&"=") = self.peek() {
                    self.next();
                    let expr = self.parse_application()?;
                    Ok(Expr::Define(tok.to_string(), Box::new(expr)))
                } else {
                    Ok(Expr::Word(tok.to_string()))
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_atom()?;
        while let Some(&tok) = self.peek() {
            if tok == ")" || tok == ";" {
                break;
            }
            let arg = self.parse_atom()?;
            expr = Expr::Application(Box::new(expr), Box::new(arg));
        }
        Ok(expr)
    }

    fn parse_expression_list(&mut self) -> Result<Expr, String> {
        let mut exprs = vec![self.parse_application()?];
        while let Some(&";") = self.peek() {
            self.next();
            exprs.push(self.parse_application()?);
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expression_list()
    }
}
