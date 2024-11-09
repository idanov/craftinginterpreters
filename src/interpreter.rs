use crate::environment::Environment;
use crate::expr::Expr;
use crate::lox_callable::{LoxClass, LoxFunction, NativeFunction};
use crate::scanner::{Literal as Lit, Token, TokenType as TT};
use crate::stmt::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    locals: HashMap<String, usize>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let locals = HashMap::new();
        let environment = globals.clone();

        globals.borrow_mut().define(
            "clock".to_string(),
            Lit::Callable(Rc::new(NativeFunction::new(
                "clock".to_string(),
                0,
                |_, _| {
                    let duration = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    Ok(Lit::Double((duration.as_millis() as f64) / 1000.0))
                },
            ))),
        );

        Interpreter {
            globals,
            locals,
            environment,
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Lit, String> {
        match expr {
            Expr::Assign(name, value) => {
                let val = self.evaluate(value)?;

                if let Some(distance) = self.locals.get(&format!("{:?}", expr)) {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name, val)
                } else {
                    self.globals.borrow_mut().assign(name, val)
                }
            }
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Call(callee, paren, arguments) => self.eval_call(callee, paren, arguments),
            Expr::Grouping(expr) => self.eval_grouping(expr),
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Logical(left, op, right) if op.token == TT::Or => {
                let res = self.evaluate(left)?;
                if Interpreter::is_truthy(&res) {
                    Ok(res)
                } else {
                    self.evaluate(right)
                }
            }
            Expr::Logical(left, _, right) => {
                let res = self.evaluate(left)?;
                if !Interpreter::is_truthy(&res) {
                    Ok(res)
                } else {
                    self.evaluate(right)
                }
            }
            Expr::Unary(op, expr) => self.eval_unary(op, expr),
            Expr::Variable(name) => self.lookup_variable(name, expr),
        }
    }

    fn lookup_variable(&mut self, name: &Token, expr: &Expr) -> Result<Lit, String> {
        if let Some(distance) = self.locals.get(&format!("{:?}", expr)) {
            self.environment.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(format!("{:?}", expr), depth);
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<Option<Lit>, String> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(None)
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<Option<Lit>, String> {
        let previous = self.environment.clone();
        self.environment = environment;
        let mut res: Result<Option<Lit>, String> = Ok(None);
        // this can be replaced in the future with iter().try_find() when added to Rust
        for stmt in statements {
            res = self.execute(stmt);
            if res.is_err() || res.as_ref().is_ok_and(|x| x.is_some()) {
                break;
            };
        }
        self.environment = previous;
        res
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Option<Lit>, String> {
        match stmt {
            Stmt::Block(statements) => {
                self.execute_block(statements, Environment::nested(self.environment.clone()))
            }
            Stmt::Class(name, _) => {
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Lit::None);
                let klass = Lit::Callable(Rc::new(LoxClass::new(name.lexeme.clone())));
                self.environment.borrow_mut().assign(name, klass)?;
                Ok(None)
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(None)
            }
            Stmt::Function(name, params, body) => {
                self.environment.borrow_mut().define(
                    name.lexeme.clone(),
                    Lit::Callable(Rc::new(LoxFunction::new(
                        name.clone(),
                        params.to_vec(),
                        body.to_vec(),
                        self.environment.clone(),
                    ))),
                );
                Ok(None)
            }
            Stmt::If(cond, then_branch, maybe_else) => {
                if Interpreter::is_truthy(&(self.evaluate(cond)?)) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = maybe_else {
                    self.execute(else_branch)
                } else {
                    Ok(None)
                }
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", value);
                Ok(None)
            }
            Stmt::Return(_, value) => Ok(Some(self.evaluate(value)?)),
            Stmt::While(cond, body) => {
                let mut res: Option<Lit> = None;
                while Interpreter::is_truthy(&(self.evaluate(cond)?)) {
                    res = self.execute(body)?;
                    if res.is_some() {
                        break;
                    }
                }
                Ok(res)
            }
            Stmt::Var(name, None) => {
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Lit::None);
                Ok(None)
            }
            Stmt::Var(name, Some(initializer)) => {
                let value = self.evaluate(initializer)?;
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);
                Ok(None)
            }
        }
    }

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Lit, String> {
        let lval = self.evaluate(left)?;
        let rval = self.evaluate(right)?;
        match (&lval, op.token, &rval) {
            (Lit::Double(lhs), TT::Minus, Lit::Double(rhs)) => Ok(Lit::Double(lhs - rhs)),
            (Lit::Double(lhs), TT::Slash, Lit::Double(rhs)) => Ok(Lit::Double(lhs / rhs)),
            (Lit::Double(lhs), TT::Star, Lit::Double(rhs)) => Ok(Lit::Double(lhs * rhs)),
            (_, TT::Minus, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::Slash, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::Star, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (Lit::Double(lhs), TT::Plus, Lit::Double(rhs)) => Ok(Lit::Double(lhs + rhs)),
            (Lit::String(lhs), TT::Plus, Lit::String(rhs)) => {
                Ok(Lit::String(lhs.to_string() + rhs))
            }
            (_, TT::Plus, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (Lit::Double(lhs), TT::Greater, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs > rhs)),
            (Lit::Double(lhs), TT::GreaterEqual, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs >= rhs)),
            (Lit::Double(lhs), TT::Less, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs < rhs)),
            (Lit::Double(lhs), TT::LessEqual, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs <= rhs)),
            (_, TT::Greater, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::GreaterEqual, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::Less, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::LessEqual, _) => Err(format!(
                "[line {}:{}] Operands must be numbers.",
                op.line, op.column
            )),
            (_, TT::EqualEqual, _) => Ok(Lit::Boolean(Interpreter::is_equal(&lval, &rval))),
            (_, TT::BangEqual, _) => Ok(Lit::Boolean(!Interpreter::is_equal(&lval, &rval))),
            _ => Ok(Lit::None),
        }
    }

    fn eval_call(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<Lit, String> {
        let callable: Lit = self.evaluate(callee)?;

        let mut args: Vec<Lit> = Vec::new();
        for arg in arguments {
            let res = self.evaluate(arg)?;
            args.push(res);
        }

        if let Lit::Callable(func) = callable {
            if args.len() != func.arity() {
                return Err(format!(
                    "[line {}:{}] Expected {} arguments but got {}.",
                    paren.line,
                    paren.column,
                    func.arity(),
                    args.len()
                ));
            }

            func.call(self, &args)
        } else {
            Err(format!(
                "[line {}:{}] Can only call functions and classes.",
                paren.line, paren.column
            ))
        }
    }

    fn eval_grouping(&mut self, expr: &Expr) -> Result<Lit, String> {
        self.evaluate(expr)
    }

    fn eval_literal(&mut self, lit: &Lit) -> Result<Lit, String> {
        Ok(lit.clone())
    }

    fn eval_unary(&mut self, op: &Token, expr: &Expr) -> Result<Lit, String> {
        let lit = self.evaluate(expr)?;
        match (op.token, &lit) {
            (TT::Minus, Lit::Double(n)) => Ok(Lit::Double(-n)),
            (TT::Minus, _) => Err(format!(
                "[line {}:{}] Operand must be a number.",
                op.line, op.column
            )),
            (TT::Bang, _) => Ok(Lit::Boolean(!Interpreter::is_truthy(&lit))),
            _ => Ok(Lit::None),
        }
    }

    fn is_truthy(lit: &Lit) -> bool {
        match lit {
            Lit::Boolean(x) => *x,
            Lit::None => false,
            _ => true,
        }
    }

    fn is_equal(left: &Lit, right: &Lit) -> bool {
        match (left, right) {
            (Lit::String(a), Lit::String(b)) => a == b,
            (Lit::Double(a), Lit::Double(b)) => a == b,
            (Lit::None, Lit::None) => true,
            (Lit::None, _) => false,
            (_, _) => false,
        }
    }
}
