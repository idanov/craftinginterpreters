use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::{Literal, Token};
use crate::stmt::Stmt;

#[derive(Debug, Clone, PartialEq, Copy)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, statements: &[Stmt]) -> Result<(), String> {
        let mut errs: Vec<String> = Vec::new();
        for statement in statements {
            if let Err(e) = self.resolve_stmt(statement) {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs.join("\n"))
        }
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Class(name, superclass, methods) => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name)?;
                self.define(name)?;

                if matches!(superclass, Some(Expr::Variable(parent)) if name.lexeme == parent
                    .lexeme)
                {
                    return Parser::error::<()>(name, "A class can't inherit from itself.");
                }
                if let Some(parent) = superclass {
                    self.current_class = ClassType::SubClass;
                    self.resolve_expr(parent)?;
                }

                if superclass.is_some() {
                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .map(|x| x.insert("super".to_string(), true));
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .map(|x| x.insert("this".to_string(), true));

                for method in methods {
                    let declaration = match method {
                        Stmt::Function(method_token, _, _) if method_token.lexeme == "init" => {
                            FunctionType::Initializer
                        }
                        _ => FunctionType::Method,
                    };
                    self.resolve_function(method, declaration)?;
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
                Ok(())
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
            Stmt::Return(keyword, expr) => match self.current_function {
                FunctionType::None => {
                    Parser::error::<()>(keyword, "Can't return from top-level code.")
                }
                FunctionType::Initializer if !matches!(expr, Expr::Literal(Literal::None)) => {
                    Parser::error::<()>(keyword, "Can't return a value from an initializer.")
                }
                _ => self.resolve_expr(expr),
            },
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
                        name,
                        "Can't read local variable in its own initializer.",
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
            Expr::Get(obj, _) => self.resolve_expr(obj),
            Expr::Set(obj, _, val) => {
                self.resolve_expr(val)?;
                self.resolve_expr(obj)?;
                Ok(())
            }
            Expr::Super(keyword, _) => {
                if self.current_class == ClassType::None {
                    Parser::error::<()>(keyword, "Can't use 'super' outside of a class.")
                } else if self.current_class != ClassType::SubClass {
                    Parser::error::<()>(keyword, "Can't use 'super' in a class with no superclass.")
                } else {
                    self.resolve_local(expr, keyword);
                    Ok(())
                }
            }
            Expr::This(keyword) => {
                if self.current_class == ClassType::None {
                    Parser::error::<()>(keyword, "Can't use 'this' outside of a class.")
                } else {
                    self.resolve_local(expr, keyword);
                    Ok(())
                }
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
                    name,
                    "Already a variable with this name in this scope.",
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
