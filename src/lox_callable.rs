use std::{
    cell::RefCell, collections::HashMap, fmt::{Debug, Display}, rc::Rc
};

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    scanner::{Literal, Token},
    stmt::Stmt,
};

pub trait LoxCallable: Display + Debug {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Literal])
        -> Result<Literal, String>;
    fn arity(&self) -> usize;
}

impl PartialEq for dyn LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, PartialEq)]
pub struct NativeFunction {
    name: String,
    arity: usize,
    callable: fn(&mut Interpreter, &[Literal]) -> Result<Literal, String>,
}

impl NativeFunction {
    pub fn new(
        name: String,
        arity: usize,
        callable: fn(&mut Interpreter, &[Literal]) -> Result<Literal, String>,
    ) -> Self {
        Self {
            name,
            arity,
            callable,
        }
    }
}
impl LoxCallable for NativeFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Literal],
    ) -> Result<Literal, String> {
        (self.callable)(interpreter, arguments)
    }

    fn arity(&self) -> usize {
        self.arity
    }
}
impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
        }
    }
}
impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Literal],
    ) -> Result<Literal, String> {
        let environment = Environment::nested(self.closure.clone());
        let it = self.params.iter().zip(arguments.iter());
        for (param, arg) in it {
            environment
                .borrow_mut()
                .define(param.lexeme.clone(), arg.clone());
        }
        let res: Option<Literal> = interpreter.execute_block(&self.body, environment)?;
        Ok(res.unwrap_or(Literal::None))
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}
impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
impl LoxCallable for LoxClass {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: &[Literal],
    ) -> Result<Literal, String> {
        let lox = RefCell::new(LoxInstance::new(Rc::new(self.clone())));
        Ok(Literal::LoxInstance(Rc::new(lox)))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: HashMap<String, Literal>

}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self { klass, fields: HashMap::new()}
    }
    pub fn get(&self, name: &Token) -> Result<Literal, String> {
        self.fields
            .get(&name.lexeme)
            .cloned()
            .ok_or(format!("[line {}:{}] Undefined property '{}'.", name.line, name.column, name.lexeme))
    }

    pub fn set(&mut self, name: &Token, val: Literal) {
        self.fields.insert(name.lexeme.clone(), val);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass)
    }
}
