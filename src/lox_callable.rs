use std::fmt::Display;

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    scanner::{Literal, Token},
    stmt::Stmt,
};

#[derive(Debug, Clone)]
pub enum LoxCallable {
    NativeFunction {
        name: String,
        arity: usize,
        callable: fn(&mut Interpreter, &[Literal]) -> Result<Literal, String>,
    },
    LoxFunction(Token, Vec<Token>, Vec<Stmt>),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            Self::NativeFunction {
                name: _,
                arity,
                callable: _,
            } => *arity,
            Self::LoxFunction(_, params, _) => params.len(),
        }
    }
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Literal],
    ) -> Result<Literal, String> {
        match self {
            Self::NativeFunction {
                name: _,
                arity: _,
                callable,
            } => (callable)(interpreter, arguments),
            Self::LoxFunction(_, params, body) => {
                let environment = Environment::nested(interpreter.globals.clone());
                let it = params.iter().zip(arguments.iter());
                for (param, arg) in it {
                    environment.borrow_mut().define(param.lexeme.clone(), arg.clone());
                }
                interpreter.execute_block(body, environment)?;
                Ok(Literal::None)
            }
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeFunction {
                name,
                arity: _,
                callable: _,
            } => write!(f, "<native fn {}>", name),
            Self::LoxFunction(name, _, _) => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
