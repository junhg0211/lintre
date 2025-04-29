use crate::ast::Expr;
use std::collections::HashMap;

pub type Env = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Closure(Vec<String>, Box<Expr>, Env),
    Unit,
}

fn substitute(expr: &Expr, var: &str, value: &Expr) -> Expr {
    match expr {
        Expr::Var(name) => {
            if name == var {
                value.clone()
            } else {
                Expr::Var(name.clone())
            }
        }
        Expr::Lambda(params, body) => {
            if params.contains(&var.to_string()) {
                Expr::Lambda(params.clone(), body.clone())
            } else {
                Expr::Lambda(params.clone(), Box::new(substitute(body, var, value)))
            }
        }
        Expr::Apply(f, arg) => {
            Expr::Apply(Box::new(substitute(f, var, value)), Box::new(substitute(arg, var, value)))
        }
        Expr::Define(name, rhs) => {
            if name == var {
                Expr::Define(name.clone(), rhs.clone())
            } else {
                Expr::Define(name.clone(), Box::new(substitute(rhs, var, value)))
            }
        }
        Expr::Sequence(exprs) => {
            Expr::Sequence(exprs.iter().map(|e| substitute(e, var, value)).collect())
        }
    }
}

pub fn eval(expr: &Expr, env: &mut Env) -> Result<Value, String> {
    match expr {
        Expr::Var(name) => {
            env.get(name).cloned().ok_or_else(|| format!("Unbound variable: {}", name))
        }
        Expr::Lambda(params, body) => {
            Ok(Value::Closure(params.clone(), body.clone(), env.clone()))
        }
        Expr::Apply(f, arg) => {
            let f_val = eval(f, env)?;
            match f_val {
                Value::Closure(mut params, body, mut closure_env) => {
                    if params.is_empty() {
                        return Err("Apply: No parameters to apply!".to_string());
                    }
                    let param = params.remove(0);
                    let arg_val = eval(arg, env)?;
                    let substituted = substitute(&body, &param, &to_expr(&arg_val));
                    if params.is_empty() {
                        eval(&substituted, &mut closure_env)
                    } else {
                        Ok(Value::Closure(params, Box::new(substituted), closure_env))
                    }
                }
                _ => Err("Apply: Not a function.".to_string()),
            }
        }
        Expr::Define(_, _) => Err("Cannot directly evaluate a Define.".to_string()),
        Expr::Sequence(exprs) => {
            if exprs.is_empty() {
                return Ok(Value::Unit);
            }
            let mut old_values = HashMap::new();
            for expr in &exprs[..exprs.len() - 1] {
                if let Expr::Define(name, rhs) = expr {
                    let val = eval(rhs, env)?;
                    if let Some(old) = env.insert(name.clone(), val) {
                        old_values.insert(name.clone(), old);
                    }
                } else {
                    eval(expr, env)?;
                }
            }
            let result = eval(&exprs[exprs.len() - 1], env);
            for (name, old) in old_values {
                env.insert(name, old);
            }
            result
        }
    }
}

fn to_expr(val: &Value) -> Expr {
    match val {
        Value::Closure(params, body, _) => Expr::Lambda(params.clone(), body.clone()),
        Value::Unit => Expr::Sequence(vec![]),
    }
}