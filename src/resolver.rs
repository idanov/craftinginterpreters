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
    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> Result<(), String> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        return Ok(());
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name);
                Ok(())
            }
            Stmt::Function(name, _, _) => {
                self.declare(name);
                self.define(name);
                self.resolve_function(statement);
                Ok(())
            }
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::If(condition, then_branch, maybe_else) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = maybe_else {
                    self.resolve_stmt(else_branch)?;
                }
                Ok(())
            }
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return(_, expr) => self.resolve_expr(expr),
            Stmt::While(condition, body) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Variable(name) => {
                if let Some(true) = self.scopes.last().and_then(|x| x.get(&name.lexeme)) {
                    return Parser::error::<()>(
                        name.clone(),
                        "Can't read local variable in its own initializer.".to_string(),
                    );
                }
                self.resolve_local(expr, name);
                Ok(())
            }
            Expr::Assign(name, value) => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
                Ok(())
            }
            _ => todo!(),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        let mut i = 0;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.borrow_mut().resolve(expr, i);
            }
            i += 1;
        }
    }

    fn resolve_function(&mut self, stmt: &Stmt) {
        if let Stmt::Function(name, params, body) = stmt {
            self.begin_scope();
            for param in params {
                self.declare(param);
                self.define(param);
            }
            self.resolve(body);
            self.end_scope();
        }
    }

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
