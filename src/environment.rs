use crate::scanner::{Literal, Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn nested(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, key: String, value: Literal) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Token) -> Result<Literal, String> {
        return self
            .values
            .get(&key.lexeme)
            .cloned()
            .or_else(|| self.enclosing.as_ref().and_then(|x| x.borrow().get(&key).ok()))
            .ok_or(format!("Undefined variable '{}'.", key.lexeme));
    }

    pub fn assign(&mut self, name: &Token, val: Literal) -> Result<Literal, String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), val.clone());
            return Ok::<Literal, String>(val);
        }

        if let Some(x) = &mut self.enclosing {
            return x.borrow_mut().assign(name, val);
        }

        return Err(format!("Undefined variable '{}'.", name.lexeme));
    }
}
