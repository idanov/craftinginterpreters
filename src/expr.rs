use crate::scanner::Literal;
use crate::scanner::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Variable(Token),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assign(name, value) => write!(f, "(= {} {})", name.lexeme, value),
            Expr::Binary(left, op, right) => write!(f, "({} {} {})", op.lexeme, left, right),
            Expr::Call(callee, _paren, arguments) => {
                let arguments = arguments.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "(call {} ({}))", callee, arguments)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Logical(left, op, right) => write!(f, "({} {} {})", op.lexeme, left, right),
            Expr::Unary(op, expr) => write!(f, "({} {})", op.lexeme, expr),
            Expr::Variable(ident) => write!(f, "{}", ident.lexeme),
        }
    }
}
