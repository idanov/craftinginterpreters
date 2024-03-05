use crate::scanner::Literal;
use crate::scanner::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary(left, op, right) => write!(f, "({} {} {})", op.lexeme, left, right),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Unary(op, expr) => write!(f, "({} {})", op.lexeme, expr),
        }
    }
}
