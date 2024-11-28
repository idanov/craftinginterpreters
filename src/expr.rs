use crate::scanner::Literal;
use crate::scanner::Token;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    This(Token),
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
                write!(f, "(call {} ({}))", callee, vec_to_string(arguments))
            }
            Expr::Get(obj, name) => write!(f, "(. {} {})", obj, name),
            Expr::Set(obj, name, val) => write!(f, "(.= {} {} {})", obj, name, val),
            Expr::This(keyword) => write!(f, "{}", keyword),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Logical(left, op, right) => write!(f, "({} {} {})", op.lexeme, left, right),
            Expr::Unary(op, expr) => write!(f, "({} {})", op.lexeme, expr),
            Expr::Variable(ident) => write!(f, "{}", ident.lexeme),
        }
    }
}

pub fn vec_to_string<T: ToString>(args: &[T]) -> String {
    args.iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
