use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    scanner::{Literal, Token},
    stmt::Stmt,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    NativeFunction {
        name: String,
        arity: usize,
        callable: fn(&mut Interpreter, &[Literal]) -> Result<Literal, String>,
    },
    LoxFunction {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            Self::NativeFunction {
                name: _,
                arity,
                callable: _,
            } => *arity,
            Self::LoxFunction {
                name: _,
                params,
                body: _,
                closure: _,
            } => params.len(),
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
            Self::LoxFunction {
                name: _,
                params,
                body,
                closure,
            } => {
                let environment = Environment::nested(closure.clone());
                let it = params.iter().zip(arguments.iter());
                for (param, arg) in it {
                    environment
                        .borrow_mut()
                        .define(param.lexeme.clone(), arg.clone());
                }
                let res: Option<Literal> = interpreter.execute_block(body, environment)?;
                Ok(res.unwrap_or(Literal::None))
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
            Self::LoxFunction {
                name,
                params: _,
                body: _,
                closure: _,
            } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
