#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Word(String),
    Words(Vec<Expr>),
    Function(Vec<String>, Box<Expr>),
    Define(String, Box<Expr>),
    Sequence(Vec<Expr>),
    Paren(Box<Expr>),
}
