use crate::ast::Expr;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
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
            Expr::Application(func_expr, arg_expr) => {
                let func_val = self.eval(*func_expr)?;
                let arg_val = self.eval(*arg_expr)?;
                match func_val {
                    Value::Closure(mut params, body, mut closure_env) => {
                        if params.is_empty() {
                            return Err("Too many arguments".to_string());
                        }
                        let param = params.remove(0);
                        closure_env.insert(param, arg_val);
                        if params.is_empty() {
                            Interpreter { env: closure_env }.eval(*body)
                        } else {
                            Ok(Value::Closure(params, body, closure_env))
                        }
                    }
                    _ => Err("Cannot apply non-function value".to_string()),
                }
            }
        }
    }

    pub fn find_name_for_value(&self, value: &Value) -> Option<String> {
        for (name, val) in &self.env {
            if val == value {
                return Some(name.clone());
            }
        }
        None
    }

}
