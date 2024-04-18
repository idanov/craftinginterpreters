use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Token;
use crate::stmt::Stmt;

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {

    fn begin_scope(&mut self) -> () {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> () {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> () {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) -> () {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }
}
