use crate::ast::Expr;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

fn normalize(expr: &Expr) -> Expr {
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

pub type Env = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Closure(Vec<String>, Box<Expr>, Env),
    Unit,
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn fresh_var_name(base: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}#{}", base, id)
}

pub fn substitute(expr: &Expr, var: &str, value: &Expr) -> Expr {
    match expr {
        Expr::Var(name) => {
            if name == var {
                value.clone()
            } else {
                Expr::Var(name.clone())
            }
        }
        Expr::Apply(f, arg) => {
            Expr::Apply(
                Box::new(substitute(f, var, value)),
                Box::new(substitute(arg, var, value)),
            )
        }
        Expr::Lambda(params, body) => {
            if params.contains(&var.to_string()) {
                let mut new_params = Vec::new();
                let mut new_body = (**body).clone();

                for p in params {
                    if p == var {
                        let fresh = fresh_var_name(p);
                        new_body = substitute(&new_body, p, &Expr::Var(fresh.clone()));
                        new_params.push(fresh);
                    } else {
                        new_params.push(p.clone());
                    }
                }

                Expr::Lambda(new_params, Box::new(substitute(&new_body, var, value)))
            } else {
                Expr::Lambda(params.clone(), Box::new(substitute(body, var, value)))
            }
        }
        Expr::Define(name, expr) => {
            Expr::Define(name.clone(), Box::new(substitute(expr, var, value)))
        }
        Expr::Sequence(exprs) => {
            Expr::Sequence(exprs.iter().map(|e| substitute(e, var, value)).collect())
        }
    }
}

fn to_expr(value: &Value) -> Expr {
    match value {
        Value::Closure(params, body, _) => Expr::Lambda(params.clone(), body.clone()),
        Value::Unit => Expr::Var("unit".to_string()),
    }
}

pub fn eval(
    expr: &Expr,
    env: &mut Env,
    trace: bool,
    step_count: &mut usize
) -> Result<Value, String> {
    match expr {
        Expr::Var(name) => {
            if trace {
                println!("Lookup variable: {}", name);
            }
            env.get(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", name))
        }
        Expr::Lambda(params, body) => {
            if trace {
                println!("Create closure: params = {:?}, body = {:?}", params, body);
            }
            Ok(Value::Closure(params.clone(), body.clone(), env.clone()))
        }
        Expr::Apply(func, arg) => {
            *step_count += 1;
            if *step_count > 10000 {
                return Err("Infinite beta reduction detected!".to_string());
            }

            let func_val = eval(func, env, trace, step_count)?;
            let arg_val = eval(arg, env, trace, step_count)?;

            match func_val {
                Value::Closure(mut params, body, mut closure_env) => {
                    if params.is_empty() {
                        return Err("Too many arguments".to_string());
                    }
                    let param = params.remove(0);

                    if trace {
                        println!("Beta reduction: substituting {} into function", param);
                        println!("Function body before: {:?}", body);
                    }

                    let substituted_body = substitute(&body, &param, &to_expr(&arg_val));

                    if trace {
                        println!("Function body after substitution: {:?}", substituted_body);
                    }

                    if params.is_empty() {
                        eval(&substituted_body, &mut closure_env, trace, step_count)
                    } else {
                        Ok(Value::Closure(params, Box::new(substituted_body), closure_env))
                    }
                }
                _ => Err("Trying to call a non-function".to_string()),
            }
        }
        Expr::Define(name, expr) => {
            let val = eval(expr, env, trace, step_count)?;
            env.insert(name.clone(), val.clone());
            if trace {
                println!("Define variable: {}", name);
            }
            Ok(val)
        }
        Expr::Sequence(exprs) => {
            let mut last = Value::Unit;
            let mut defined_vars = Vec::new();
            let mut old_values = HashMap::new();

            for expr in exprs {
                match expr {
                    Expr::Define(name, rhs) => {
                        let val = eval(rhs, env, trace, step_count)?;
                        if let Some(old) = env.get(name).cloned() {
                            old_values.insert(name.clone(), old);
                        } else {
                            defined_vars.push(name.clone());
                        }
                        env.insert(name.clone(), val);
                        if trace {
                            println!("Define (in block) variable: {}", name);
                        }
                    }
                    _ => {
                        last = eval(expr, env, trace, step_count)?;
                    }
                }
            }

            for name in defined_vars {
                env.remove(&name);
                if trace {
                    println!("Remove variable after block: {}", name);
                }
            }
            for (name, old_val) in old_values {
                env.insert(name.clone(), old_val);
                if trace {
                    println!("Restore variable after block: {}", name);
                }
            }

            Ok(last)
        }
    }
}
