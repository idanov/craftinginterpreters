use crate::{expr::Expr, scanner::Token};
use crate::expr::vec_to_string;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Option<Expr>),
    While(Expr, Box<Stmt>),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Block(statements) => write!(f, "(block {})", vec_to_string(&statements)),
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::Function(name, params, body) => write!(f, "(fun {} ({}) ({}))", name.lexeme, vec_to_string(&params), vec_to_string(&body)),
            Stmt::If(cond, then_branch, Some(else_branch)) => write!(f, "(if {} (then {}) (else {}))", cond, then_branch, else_branch),
            Stmt::If(cond, then_branch, None) => write!(f, "(if {} (then {}))", cond, then_branch),
            Stmt::Print(expr) => write!(f, "(print {})", expr),
            Stmt::Return(_token, value) => write!(f, "(return {})", value),
            Stmt::Var(token, Some(expr)) => write!(f, "(var {} {})", token.lexeme, expr),
            Stmt::Var(token, None) => write!(f, "(var {} nil)", token.lexeme),
            Stmt::While(cond, body) => write!(f, "(while {} (body {}))", cond, body),
        }
    }
}
