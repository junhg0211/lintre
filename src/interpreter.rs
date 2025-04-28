use crate::ast::Expr;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Value {
    Word(String),
    Closure(Vec<String>, Box<Expr>, Env),
}

type Env = HashMap<String, Value>;

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: HashMap::new() }
    }

    pub fn eval(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Word(name) => {
                self.env.get(&name).cloned().ok_or_else(|| format!("Unbound variable: {}", name))
            }
            Expr::Function(params, body) => {
                Ok(Value::Closure(params, body, self.env.clone()))
            }
            Expr::Define(name, body) => {
                let val = self.eval(*body)?;
                self.env.insert(name.clone(), val.clone());
                Ok(val)
            }
            Expr::Sequence(exprs) => {
                let mut last_val = None;
                for expr in exprs {
                    last_val = Some(self.eval(expr)?);
                }
                last_val.ok_or_else(|| "Empty sequence".to_string())
            }
        }
    }
}