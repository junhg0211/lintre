use std::collections::HashMap;

use crate::ast::Expr;

pub type Env = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Closure(Vec<String>, Box<Expr>, Env),
    Unit,
}

pub fn eval(expr: &Expr, env: &mut Env, trace: bool, step_count: &mut usize) -> Result<Value, String> {
    *step_count += 1;
    if *step_count > 10000 {
        return Err("Infinite beta reduction detected!".to_string());
    }

    match expr {
        Expr::Var(name) => {
            if let Some(val) = env.get(name) {
                Ok(val.clone())
            } else {
                Err(format!("Unbound variable: {}", name))
            }
        }
        Expr::Lambda(params, body) => {
            Ok(Value::Closure(params.clone(), body.clone(), env.clone()))
        }
        Expr::Apply(f, arg) => {
            let f_val = eval(f, env, trace, step_count)?;
            match f_val {
                Value::Closure(mut params, body, mut closure_env) => {
                    if params.is_empty() {
                        return Err("Apply: No parameters to apply!".to_string());
                    }
                    let param_name = params.remove(0);
                    let arg_val = eval(arg, env, trace, step_count)?;

                    // substitute param_name with arg_val in body
                    let substituted_body = substitute(&body, &param_name, &to_expr(&arg_val));

                    if params.is_empty() {
                        eval(&substituted_body, &mut closure_env, trace, step_count)
                    } else {
                        Ok(Value::Closure(params, Box::new(substituted_body), closure_env))
                    }
                }
                _ => Err("Apply: Function is not a closure".to_string()),
            }
        }
        Expr::Define(name, expr) => {
            let val = eval(expr, env, trace, step_count)?;
            env.insert(name.clone(), val);
            Ok(Value::Unit)
        }
        Expr::Sequence(exprs) => {
            let mut last = Value::Unit;
            for expr in exprs {
                last = eval(expr, env, trace, step_count)?;
            }
            Ok(last)
        }
    }
}

fn substitute(expr: &Expr, var: &str, replacement: &Expr) -> Expr {
    match expr {
        Expr::Var(name) => {
            if name == var {
                replacement.clone()
            } else {
                Expr::Var(name.clone())
            }
        }
        Expr::Lambda(params, body) => {
            if params.contains(&var.to_string()) {
                Expr::Lambda(params.clone(), body.clone()) // variable shadowing
            } else {
                Expr::Lambda(params.clone(), Box::new(substitute(body, var, replacement)))
            }
        }
        Expr::Apply(f, arg) => {
            Expr::Apply(
                Box::new(substitute(f, var, replacement)),
                Box::new(substitute(arg, var, replacement)),
            )
        }
        Expr::Define(name, expr) => {
            if name == var {
                Expr::Define(name.clone(), expr.clone())
            } else {
                Expr::Define(name.clone(), Box::new(substitute(expr, var, replacement)))
            }
        }
        Expr::Sequence(exprs) => {
            Expr::Sequence(exprs.iter().map(|e| substitute(e, var, replacement)).collect())
        }
    }
}

fn to_expr(value: &Value) -> Expr {
    match value {
        Value::Closure(params, body, _) => Expr::Lambda(params.clone(), body.clone()),
        Value::Unit => Expr::Sequence(vec![]), // Unit 표현은 비어 있는 Sequence로
    }
}

pub fn normalize(expr: &Expr) -> Expr {
    fn normalize_rec(expr: &Expr, var_map: &mut HashMap<String, String>, counter: &mut usize) -> Expr {
        match expr {
            Expr::Var(name) => {
                if let Some(new_name) = var_map.get(name) {
                    Expr::Var(new_name.clone())
                } else {
                    Expr::Var(name.clone())
                }
            }
            Expr::Lambda(params, body) => {
                let mut new_params = Vec::new();
                for param in params {
                    let new_name = format!("v{}", *counter);
                    *counter += 1;
                    var_map.insert(param.clone(), new_name.clone());
                    new_params.push(new_name);
                }
                let new_body = normalize_rec(body, var_map, counter);
                for param in params {
                    var_map.remove(param);
                }
                Expr::Lambda(new_params, Box::new(new_body))
            }
            Expr::Apply(f, arg) => {
                Expr::Apply(
                    Box::new(normalize_rec(f, var_map, counter)),
                    Box::new(normalize_rec(arg, var_map, counter)),
                )
            }
            Expr::Define(name, expr) => {
                Expr::Define(name.clone(), Box::new(normalize_rec(expr, var_map, counter)))
            }
            Expr::Sequence(exprs) => {
                Expr::Sequence(exprs.iter().map(|e| normalize_rec(e, var_map, counter)).collect())
            }
        }
    }

    let mut var_map = HashMap::new();
    let mut counter = 0;
    normalize_rec(expr, &mut var_map, &mut counter)
}