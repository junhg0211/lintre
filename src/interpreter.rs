use crate::ast::Expr;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Env = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub enum Value {
    Closure(Vec<String>, Box<Expr>, Env),
    Unit,
}

// 전역 counter: fresh 변수 이름 만들 때 사용
static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn fresh_var_name(base: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}#{}", base, id)
}

/// 알파 변환과 substitution 수행
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

/// eval 함수
pub fn eval(expr: &Expr, env: &mut Env, trace: bool, seen: &mut HashSet<(Vec<String>, Expr)>) -> Result<Value, String> {
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
            let func_val = eval(func, env, trace, seen)?;
            let arg_val = eval(arg, env, trace, seen)?;

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

                    let fingerprint = (params.clone(), substituted_body.clone());
                    if !seen.insert(fingerprint) {
                        return Err("Infinite beta reduction detected!".to_string());
                    }

                    if params.is_empty() {
                        eval(&substituted_body, &mut closure_env, trace, seen)
                    } else {
                        Ok(Value::Closure(params, Box::new(substituted_body), closure_env))
                    }
                }
                _ => Err("Trying to call a non-function".to_string()),
            }
        }
        Expr::Define(name, expr) => {
            let val = eval(expr, env, trace, seen)?;
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
                        let val = eval(rhs, env, trace, seen)?;

                        // 기존 값이 있으면 기억해놓자
                        if let Some(old) = env.get(name).cloned() {
                            old_values.insert(name.clone(), old);
                        } else {
                            defined_vars.push(name.clone()); // 새로 정의된 것만 따로 기억
                        }

                        env.insert(name.clone(), val);

                        if trace {
                            println!("Define (in block) variable: {}", name);
                        }
                    }
                    _ => {
                        last = eval(expr, env, trace, seen)?;
                    }
                }
            }

            // 블록 끝났을 때
            for name in defined_vars {
                env.remove(&name); // 새로 만든 건 그냥 삭제
                if trace {
                    println!("Remove variable after block: {}", name);
                }
            }
            for (name, old_val) in old_values {
                env.insert(name.clone(), old_val); // 덮어쓴 건 복구
                if trace {
                    println!("Restore variable after block: {}", name);
                }
            }

            Ok(last)
        }
    }
}
