use crate::ast::Expr;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 새로운 변수 이름을 만들어주는 함수
fn fresh_var_name(base: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}#{}", base, id)
}

/// substitute 함수: expr 안의 var 이름을 value로 대체한다.
/// 만약 내부 람다 매개변수와 var가 겹치면, fresh 이름으로 알파 변환을 수행한다.
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
                // 충돌 발생. 알파 변환 수행
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
                // 충돌 없으면 그냥 재귀적으로 치환
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

pub fn eval_document(expr: &Expr, env: &mut Env, trace_mode: &TraceMode) -> Result<Value, String> {
    let mut steps = 0;

    match expr {
        Expr::Sequence(exprs) => {
            let mut last = Value::Unit;
            for (i, expr) in exprs.iter().enumerate() {
                let is_last = i == exprs.len() - 1;
                let trace = match trace_mode {
                    TraceMode::None => false,
                    TraceMode::Last => is_last,
                    TraceMode::All => true,
                };
                last = eval(expr, env, trace, &mut steps)?;
            }
            Ok(last)
        }
        _ => {
            eval(expr, env, matches!(trace_mode, TraceMode::Last | TraceMode::All), &mut steps)
        }
    }
}

const MAX_STEPS: usize = 10000;

pub fn eval(expr: &Expr, env: &mut Env, trace: bool, steps: &mut usize) -> Result<Value, String> {
    if *steps > MAX_STEPS {
        return Err("Infinite beta reduction detected!".to_string());
    }

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
            *steps += 1; // <-- 여기서 스텝 수 증가

            let func_val = eval(func, env, trace, steps)?;
            let arg_val = eval(arg, env, trace, steps)?;

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
                        eval(&substituted_body, &mut closure_env, trace, steps)
                    } else {
                        Ok(Value::Closure(params, Box::new(substituted_body), closure_env))
                    }
                }
                _ => Err("Trying to call a non-function".to_string()),
            }
        }
        Expr::Define(name, expr) => {
            let val = eval(expr, env, trace, steps)?;
            env.insert(name.clone(), val.clone());
            if trace {
                println!("Define variable: {}", name);
            }
            Ok(val)
        }
        Expr::Sequence(exprs) => {
            let mut last = Value::Unit;
            for expr in exprs {
                last = eval(expr, env, trace, steps)?;
            }
            Ok(last)
        }
    }
}
