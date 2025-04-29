#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Var(String),
    Lambda(Vec<String>, Box<Expr>),
    Apply(Box<Expr>, Box<Expr>),
    Define(String, Box<Expr>),
    Sequence(Vec<Expr>),
}
