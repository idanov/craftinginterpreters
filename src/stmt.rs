use crate::{expr::Expr, scanner::Token};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Option<Expr>),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Block(statements) => {
                write!(f, "(block")?;
                for stmt in statements {
                    write!(f, " {}", stmt)?;
                }
                write!(f, ")")?;
                Ok(())
            },
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::If(cond, then_branch, Some(else_branch)) => write!(f, "(if {} (then {}) (else {}))", cond, then_branch, else_branch),
            Stmt::If(cond, then_branch, None) => write!(f, "(if {} (then {}))", cond, then_branch),
            Stmt::Print(expr) => write!(f, "(print {})", expr),
            Stmt::Var(token, Some(expr)) => write!(f, "(var {} {})", token.lexeme, expr),
            Stmt::Var(token, None) => write!(f, "(var {} nil)", token.lexeme),
        }
    }
}
