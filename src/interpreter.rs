use crate::ast::Expr;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Value {
    Closure(Vec<String>, Box<Expr>, Env),
    Word(String),
}

type Env = HashMap<String, Value>;

pub struct Interpreter {
    env: Env,
    debug: bool,
    previous_states: HashSet<String>,
    name_counter: HashMap<String, usize>,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        Self {
            env: HashMap::new(),
            debug,
            previous_states: HashSet::new(),
            name_counter: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Word(name) => {
                if let Some(v) = self.env.get(&name) {
                    Ok(v.clone())
                } else {
                    Ok(Value::Word(name))
                }
            }
            Expr::Words(mut words) => {
                if words.is_empty() {
                    return Err("Empty Words expression.".to_string());
                }
                let mut func = self.eval(words.remove(0))?;
                for word in words {
                    let arg = self.eval(word)?;
                    func = self.apply(func, arg)?;
                }
                Ok(func)
            }
            Expr::Function(params, body) => {
                let fresh_params = params.iter()
                    .map(|p| self.fresh_name(p))
                    .collect::<Vec<_>>();

                let mut mapping = HashMap::new();
                for (old, new) in params.iter().zip(fresh_params.iter()) {
                    mapping.insert(old.clone(), new.clone());
                }

                let renamed_body = self.rename(*body, &mapping);

                Ok(Value::Closure(fresh_params, Box::new(renamed_body), self.env.clone()))
            }
            Expr::Define(name, body) => {
                let val = self.eval(*body)?;
                self.env.insert(name.clone(), val.clone());
                Ok(val)
            }
            Expr::Sequence(exprs) => {
                let mut last_expr = None;
                for expr in exprs {
                    match expr {
                        Expr::Define(name, body) => {
                            let val = self.eval(*body)?;
                            self.env.insert(name, val);
                        }
                        _ => {
                            last_expr = Some(expr);
                        }
                    }
                }
                if let Some(e) = last_expr {
                    self.eval(e)
                } else {
                    Ok(Value::Word("()".to_string()))
                }
            }
            Expr::Paren(inner) => self.eval(*inner),
        }
    }

    fn apply(&mut self, func: Value, arg: Value) -> Result<Value, String> {
        match func {
            Value::Closure(mut params, body, mut closure_env) => {
                if params.is_empty() {
                    return Err("No parameter left to apply!".to_string());
                }
                let param = params.remove(0);

                closure_env.insert(param, arg);

                if self.debug {
                    println!("--- β-reduction step ---");
                    println!("Applying: {}", self.pretty_expr(&body));
                    println!("With environment:");
                    for (k, v) in &closure_env {
                        println!("  {} = {}", k, self.pretty_value(v));
                    }
                    println!();
                }

                let state_key = format!("{} {:?}", self.pretty_expr(&body), closure_env.keys());
                if self.previous_states.contains(&state_key) {
                    return Err("무한 β-축약 루프 감지!".to_string());
                }
                self.previous_states.insert(state_key);

                if params.is_empty() {
                    let mut next = Interpreter::new(self.debug);
                    next.env = closure_env;
                    next.name_counter = self.name_counter.clone();
                    next.previous_states = self.previous_states.clone();
                    next.eval(*body)
                } else{
                    Ok(Value::Closure(params, body, closure_env))
                }
            }
            _ => Err("Trying to apply non-function!".to_string()),
        }
    }

    pub fn format_result(&self, value: &Value) -> String {
        match value {
            Value::Word(w) => w.clone(),
            Value::Closure(params, body, _) => {
                format!("(λ{} . {})", params.join(" "), self.pretty_expr(body))
            }
        }
    }

    fn pretty_value(&self, v: &Value) -> String {
        match v {
            Value::Word(w) => w.clone(),
            Value::Closure(params, _body, _) => {
                format!("(λ{} . ...)", params.join(" "))
            }
        }
    }

    fn pretty_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Word(w) => w.clone(),
            Expr::Words(ws) => ws.iter()
                .map(|e| self.pretty_expr(e))
                .collect::<Vec<_>>()
                .join(" "),
            Expr::Function(params, body) => {
                format!("(λ{} . {})", params.join(" "), self.pretty_expr(body))
            }
            Expr::Define(name, body) => {
                format!("{} = {}", name, self.pretty_expr(body))
            }
            Expr::Sequence(seq) => {
                seq.iter()
                    .map(|e| self.pretty_expr(e))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
            Expr::Paren(inner) => {
                format!("({})", self.pretty_expr(inner))
            }
        }
    }

    fn fresh_name(&mut self, base: &str) -> String {
        let count = self.name_counter.entry(base.to_string()).or_insert(0);
        *count += 1;
        format!("{}${}", base, count)
    }

    fn rename(&mut self, expr: Expr, mapping: &HashMap<String, String>) -> Expr {
        match expr {
            Expr::Word(w) => {
                if let Some(new_w) = mapping.get(&w) {
                    Expr::Word(new_w.clone())
                } else {
                    Expr::Word(w)
                }
            }
            Expr::Words(ws) => {
                Expr::Words(ws.into_iter()
                    .map(|e| self.rename(e, mapping))
                    .collect())
            }
            Expr::Function(params, body) => {
                let mut new_mapping = mapping.clone();
                let new_params = params.into_iter()
                    .map(|p| {
                        let fresh = self.fresh_name(&p);
                        new_mapping.insert(p, fresh.clone());
                        fresh
                    })
                    .collect();
                let new_body = self.rename(*body, &new_mapping);
                Expr::Function(new_params, Box::new(new_body))
            }
            Expr::Define(name, body) => {
                Expr::Define(name, Box::new(self.rename(*body, mapping)))
            }
            Expr::Sequence(seq) => {
                Expr::Sequence(seq.into_iter()
                    .map(|e| self.rename(e, mapping))
                    .collect())
            }
            Expr::Paren(inner) => {
                Expr::Paren(Box::new(self.rename(*inner, mapping)))
            }
        }
    }
}
