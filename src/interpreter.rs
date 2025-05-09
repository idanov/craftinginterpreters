use crate::environment::Environment;
use crate::expr::Expr;
use crate::lox_callable::{LoxCallable, LoxClass, LoxFunction, LoxInstance, NativeFunction};
use crate::scanner::{Literal as Lit, Literal, Token, TokenType as TT};
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
            "clock",
            Lit::Callable(LoxCallable::NativeFunction(Rc::new(NativeFunction::new(
                "clock",
                0,
                |_, _| {
                    let duration = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    Ok(Lit::Double((duration.as_millis() as f64) / 1000.0))
                },
            )))),
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
                    let mut env = self.environment.borrow_mut();
                    env.assign_at(*distance, name, val)
                } else {
                    self.globals.borrow_mut().assign(name, val)
                }
            }
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Call(callee, paren, arguments) => self.eval_call(callee, paren, arguments),
            Expr::Get(obj, name) => self.eval_get(obj, name),
            Expr::Set(obj, name, val) => self.eval_set(obj, name, val),
            Expr::Super(keyword, method) => {
                let distance = *self.locals.get(&format!("{:?}", expr)).unwrap_or(&0);
                let superclass = self
                    .environment
                    .borrow()
                    .get_at(distance, &keyword.lexeme)?;
                let instance = self.environment.borrow().get_at(distance - 1, "this")?;
                let res =
                    if let (Lit::Callable(LoxCallable::LoxClass(parent)), Lit::LoxInstance(obj)) =
                        (superclass, instance)
                    {
                        parent
                            .find_method(&method.lexeme)
                            .map(|m| LoxCallable::LoxFunction(m.bind(obj.clone())))
                            .map(Lit::Callable)
                    } else {
                        None
                    };
                res.ok_or(format!(
                    "[line {}:{}] Undefined property '{}'.",
                    method.line, method.column, method.lexeme
                ))
            }
            Expr::This(keyword) => self.lookup_variable(keyword, expr),
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

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<Option<Lit>, String> {
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
            Stmt::Class(name, superclass, class_methods) => {
                let parent = superclass
                    .clone()
                    .map(|x| self.evaluate(&x))
                    .transpose()?
                    .map(|x| match x {
                        Literal::Callable(LoxCallable::LoxClass(class)) => Ok(Rc::clone(&class)),
                        _ => Err(format!(
                            "[line {}:{}] Superclass must be a class.",
                            name.line, name.column
                        )),
                    })
                    .transpose()?;

                self.environment
                    .borrow_mut()
                    .define(&name.lexeme, Lit::None);

                if let Some(super_ref) = &parent {
                    self.environment = Environment::nested(self.environment.clone());
                    self.environment.borrow_mut().define(
                        "super",
                        Literal::Callable(LoxCallable::LoxClass(super_ref.clone())),
                    );
                }

                let mut methods: HashMap<String, Rc<LoxFunction>> = HashMap::new();
                for x in class_methods {
                    if let Stmt::Function(name, params, body) = x {
                        let method = LoxFunction::new(
                            name.clone(),
                            params.to_vec(),
                            body.to_vec(),
                            self.environment.clone(),
                            name.lexeme == "init",
                        );
                        methods.insert(name.lexeme.clone(), Rc::new(method));
                    }
                }

                let klass = Lit::Callable(LoxCallable::LoxClass(Rc::new(LoxClass::new(
                    &name.lexeme,
                    parent,
                    methods,
                ))));

                if superclass.is_some() {
                    let ancestor = self.environment.borrow().ancestor(0);
                    self.environment = ancestor;
                }

                self.environment.borrow_mut().assign(name, klass)?;
                Ok(None)
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(None)
            }
            Stmt::Function(name, params, body) => {
                self.environment.borrow_mut().define(
                    &name.lexeme,
                    Lit::Callable(LoxCallable::LoxFunction(Rc::new(LoxFunction::new(
                        name.clone(),
                        params.to_vec(),
                        body.to_vec(),
                        self.environment.clone(),
                        false,
                    )))),
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
                if let Lit::String(val) = value {
                    println!("{}", val);
                } else {
                    println!("{}", value);
                }
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
                    .define(&name.lexeme, Lit::None);
                Ok(None)
            }
            Stmt::Var(name, Some(initializer)) => {
                let value = self.evaluate(initializer)?;
                self.environment.borrow_mut().define(&name.lexeme, value);
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
                Ok(Lit::String(format!("{}{}", lhs, rhs)))
            }
            (Lit::String(lhs), TT::Plus, Lit::Double(rhs)) => {
                Ok(Lit::String(format!("{}{}", lhs, rhs)))
            }
            (Lit::Double(lhs), TT::Plus, Lit::String(rhs)) => {
                Ok(Lit::String(format!("{}{}", lhs, rhs)))
            }
            (_, TT::Plus, _) => Err(format!(
                "[line {}:{}] Operands must be two numbers or two strings.",
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
        arguments: &[Expr],
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

    fn eval_get(&mut self, obj: &Expr, name: &Token) -> Result<Lit, String> {
        let object = self.evaluate(obj)?;
        if let Lit::LoxInstance(inst) = object {
            LoxInstance::get(inst, name)
        } else {
            Err(format!(
                "[line {}:{}] Only instances have properties.",
                name.line, name.column
            ))
        }
    }

    fn eval_set(&mut self, obj: &Expr, name: &Token, val: &Expr) -> Result<Lit, String> {
        let object = self.evaluate(obj)?;
        if let Lit::LoxInstance(inst) = object {
            let value = self.evaluate(val)?;
            inst.borrow_mut().set(name, value.clone());
            Ok(value)
        } else {
            Err(format!(
                "[line {}:{}] Only instances have fields.",
                name.line, name.column
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
            (Lit::Boolean(a), Lit::Boolean(b)) => a == b,
            (Lit::String(a), Lit::String(b)) => a == b,
            (Lit::Double(a), Lit::Double(b)) => a == b,
            (Lit::None, Lit::None) => true,
            (Lit::None, _) => false,
            (Lit::Callable(a), Lit::Callable(b)) => a == b,
            (Lit::LoxInstance(a), Lit::LoxInstance(b)) => a == b,
            (_, _) => false,
        }
    }
}
