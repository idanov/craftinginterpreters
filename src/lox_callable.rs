use std::{
    cell::RefCell, collections::HashMap, fmt::{Debug, Display}, rc::Rc, ptr
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
        ptr::eq(self as *const Self, other as *const Self)
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
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Rc<LoxFunction> {
        let environment = Environment::nested(self.closure.clone());
        environment.borrow_mut().define("this".to_string(), Literal::LoxInstance(Rc::clone(&instance)));
        Rc::new(LoxFunction::new(self.name.clone(), self.params.to_vec(), self.body.to_vec(),
        environment, self.is_initializer))
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
        if self.is_initializer {
            self.closure.borrow_mut().get_at(0, "this")
        } else {
            Ok(res.unwrap_or(Literal::None))
        }
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
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).cloned()
    }
}
impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Literal],
    ) -> Result<Literal, String> {
        let lox = Rc::new(RefCell::new(LoxInstance::new(Rc::new(self.clone()))));
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(lox.clone()).call(interpreter, arguments)?;
        }
        Ok(Literal::LoxInstance(lox))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
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
    pub fn get(obj: Rc<RefCell<Self>>, name: &Token) -> Result<Literal, String> {
        let lambda = || {obj.borrow().klass.find_method(&name.lexeme)
            .map(|x| Literal::Callable(x.bind(Rc::clone(&obj))))};
        obj.borrow().fields
            .get(&name.lexeme)
            .cloned()
            .or_else(lambda)
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
