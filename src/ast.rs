#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Word(String),
    Function(Vec<String>, Box<Expr>),
    Define(String, Box<Expr>),
    Sequence(Vec<Expr>),
    Application(Box<Expr>, Box<Expr>),
}
