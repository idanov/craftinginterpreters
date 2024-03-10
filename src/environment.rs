use crate::scanner::{Literal, Token};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, key: String, value: Literal) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Token) -> Result<Literal, String> {
        self.values
            .get(&key.lexeme)
            .cloned()
            .ok_or(format!("Undefined variable '{}'.", key.lexeme))
    }

    pub fn assign(&mut self, name: &Token, val: Literal) -> Result<Literal, String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), val.clone());
            return Ok::<Literal, String>(val);
        }
        return Err(format!("Undefined variable '{}'.", name.lexeme));
    }
}
