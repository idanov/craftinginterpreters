use crate::scanner::{Literal, Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, PartialEq)]
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

    pub fn nested(enclosing: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, key: String, value: Literal) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Token) -> Result<Literal, String> {
        return self
            .values
            .get(&key.lexeme)
            .cloned()
            .or_else(|| {
                self.enclosing
                    .as_ref()
                    .and_then(|x| x.borrow().get(&key).ok())
            })
            .ok_or(format!("Undefined variable '{}'.", key.lexeme));
    }

    pub fn get_at(&self, distance: usize, name: &String) -> Result<Literal, String> {
        if distance > 0 {
            self.ancestor(distance).borrow().values.get(name).cloned()
        } else {
            self.values.get(name).cloned()
        }
        .ok_or(format!("Undefined variable '{}'.", name))
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        val: Literal,
    ) -> Result<Literal, String> {
        if distance > 0 {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.lexeme.clone(), val.clone());
        } else {
            self.values.insert(name.lexeme.clone(), val.clone());
        }
        Ok(val)
    }

    pub fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut current = self.enclosing.clone().expect("No parent environment");

        for _ in 1..distance {
            current = {
                let borrowed = current.borrow();
                borrowed
                    .enclosing
                    .clone()
                    .expect("No further parent environment")
            };
        }

        current
    }

    pub fn assign(&mut self, name: &Token, val: Literal) -> Result<Literal, String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), val.clone());
            return Ok(val);
        }

        if let Some(x) = &mut self.enclosing {
            return x.borrow_mut().assign(name, val);
        }

        return Err(format!("Undefined variable '{}'.", name.lexeme));
    }
}
