use crate::tokenizer::Token;
use crate::ast::Expr;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() { self.pos += 1; }
        tok
    }

    pub fn parse_document(&mut self) -> Result<Expr, String> {
        let mut exprs = Vec::new();
        exprs.push(self.parse_expression()?);

        while let Some(Token::Semicolon) = self.peek() {
            self.next();
            exprs.push(self.parse_expression()?);
        }

        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_document()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Expected ')'".to_string()),
                }
            }
            Some(Token::Word(_)) => {
                if let Some(Token::Equal) = self.tokens.get(self.pos + 1) {
                    self.parse_define()
                } else {
                    self.parse_apply()
                }
            }
            Some(Token::Lambda) => {
                self.parse_function()
            }
            _ => Err("Unexpected token in expression".to_string()),
        }
    }

    fn parse_define(&mut self) -> Result<Expr, String> {
        let name = match self.next() {
            Some(Token::Word(name)) => name.clone(),
            _ => return Err("Expected word in define".to_string()),
        };
        match self.next() {
            Some(Token::Equal) => (),
            _ => return Err("Expected '=' after word".to_string()),
        }
        let expr = self.parse_expression()?;
        Ok(Expr::Define(name, Box::new(expr)))
    }

    fn parse_apply(&mut self) -> Result<Expr> {
        let mut expr = self.parse_function()?;

        while matches!(self.peek(), Some(Token::Word(_)) | Some(Token::Lambda) | Some(Token::LParen)) {
            let arg = self.parse_function()?;
            expr = Expr::Apply(Box::new(expr), Box::new(arg));
        }

        Ok(expr)
    }

    fn parse_function(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Word(name)) => {
                self.next();
                Ok(Expr::Var(name))
            }
            Some(Token::Lambda) => {
                self.next();
                let mut params = Vec::new();
                while let Some(Token::Word(name)) = self.peek().cloned() {
                    self.next();
                    params.push(name);
                }
                match self.next() {
                    Some(Token::Dot) => (),
                    _ => return Err("Expected '.' after lambda parameters".to_string()),
                }
                let body = self.parse_expression()?;
                Ok(Expr::Lambda(params, Box::new(body)))
            }
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_document()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Expected ')'".to_string()),
                }
            }
            _ => Err("Unexpected token in function".to_string()),
        }
    }
}
