use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Token;
use crate::stmt::Stmt;

#[derive(Debug, Clone, PartialEq, Copy)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> Result<(), String> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Class(name, _) => {
                self.declare(name)?;
                self.define(name)
            }
            Stmt::Var(name, initializer) => {
                self.declare(name)?;
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name)
            }
            Stmt::Function(name, _, _) => {
                self.declare(name)?;
                self.define(name)?;
                self.resolve_function(statement, FunctionType::Function)
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
            Stmt::Return(keyword, expr) => {
                if matches!(self.current_function, FunctionType::None) {
                    Parser::error::<()>(
                        keyword.clone(),
                        "Can't return from top-level code.".to_string(),
                    )
                } else {
                    self.resolve_expr(expr)
                }
            }
            Stmt::While(condition, body) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Variable(name) => {
                if let Some(false) = self.scopes.last().and_then(|x| x.get(&name.lexeme)) {
                    return Parser::error::<()>(
                        name.clone(),
                        "Can't read local variable in its own initializer.".to_string(),
                    );
                }
                self.resolve_local(expr, name);
                Ok(())
            }
            Expr::Assign(name, value) => {
                self.resolve_expr(value)?;
                self.resolve_local(expr, name);
                Ok(())
            }
            Expr::Binary(left, _, right) => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call(callee, _, args) => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            Expr::Grouping(expr) => self.resolve_expr(expr),
            Expr::Literal(_) => Ok(()),
            Expr::Logical(left, _, right) => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            }
            Expr::Unary(_, right) => self.resolve_expr(right),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.borrow_mut().resolve(expr, i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, stmt: &Stmt, type_: FunctionType) -> Result<(), String> {
        if let Stmt::Function(_, params, body) = stmt {
            let enclosing_function = self.current_function;
            self.current_function = type_;
            self.begin_scope();
            for param in params {
                self.declare(param)?;
                self.define(param)?;
            }
            self.resolve(body)?;
            self.end_scope();
            self.current_function = enclosing_function;
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Parser::error::<()>(
                    name.clone(),
                    "Already a variable with this name in this scope.".to_string(),
                );
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
        Ok(())
    }
}
