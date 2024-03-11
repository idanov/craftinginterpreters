use crate::scanner::{Literal, Token};
use std::{collections::HashMap, mem};

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn nested(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn detach(&mut self) -> Option<Box<Self>> {
        return mem::replace(&mut self.enclosing, None);
    }

    pub fn define(&mut self, key: String, value: Literal) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Token) -> Result<Literal, String> {
        return self
            .values
            .get(&key.lexeme)
            .cloned()
            .or_else(|| self.enclosing.as_ref().and_then(|x| x.get(&key).ok()))
            .ok_or(format!("Undefined variable '{}'.", key.lexeme));
    }

    pub fn assign(&mut self, name: &Token, val: Literal) -> Result<Literal, String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), val.clone());
            return Ok::<Literal, String>(val);
        }

        if let Some(x) = &mut self.enclosing {
            return x.assign(name, val);
        }

        return Err(format!("Undefined variable '{}'.", name.lexeme));
    }
}
