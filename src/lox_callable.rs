use std::fmt::Display;

use crate::{interpreter::Interpreter, scanner::Literal};

#[derive(Debug, Clone)]
pub struct LoxCallable {
    pub name: String,
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Literal>) -> Result<Literal, String>,
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        self.arity
    }
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Literal>) -> Result<Literal, String> {
        (self.callable)(interpreter, arguments)
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "callable")
    }
}
