#[derive(Debug, Clone)]
pub enum Expr {
    Word(String),
    Function(Vec<String>, Box<Expr>),
    Define(String, Box<Expr>),
    Sequence(Vec<Expr>),
}
