use std::fmt::Display;

use crate::{interpreter::Interpreter, scanner::Literal};

#[derive(Debug, Clone)]
pub struct LoxCallable {

}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        todo!()
    }
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Literal>) -> Result<Literal, String> {
        todo!()
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "callable")
    }
}
