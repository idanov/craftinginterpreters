use crate::expr::Expr;
use crate::scanner::{Token, Literal as Lit, TokenType as TT};

pub struct Interpreter {
}


impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Lit, String> {
        match expr {
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Grouping(expr) => self.eval_grouping(expr),
            Expr::Literal(lit) => self.eval_literal(&lit),
            Expr::Unary(op, expr) => self.eval_unary(op, expr),
        }
    }

    pub fn interpret(&mut self, expr: &Expr) {
        self.evaluate(expr);
    }

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Lit, String> {
        let lval = self.evaluate(left)?;
        let rval = self.evaluate(right)?;
        match (&lval, op.token, &rval) {
            (Lit::Double(lhs), TT::Minus, Lit::Double(rhs)) => Ok(Lit::Double(lhs - rhs)),
            (Lit::Double(lhs), TT::Slash, Lit::Double(rhs)) => Ok(Lit::Double(lhs / rhs)),
            (Lit::Double(lhs), TT::Star, Lit::Double(rhs)) => Ok(Lit::Double(lhs * rhs)),
            (_, TT::Minus, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::Slash, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::Star, _) => Err("Operands must be numbers.".to_string()),
            (Lit::Double(lhs), TT::Plus, Lit::Double(rhs)) => Ok(Lit::Double(lhs + rhs)),
            (Lit::String(lhs), TT::Plus, Lit::String(rhs)) => Ok(Lit::String(lhs.to_string() + rhs)),
            (_, TT::Plus, _) => Err("Operands must be two numbers or two strings.".to_string()),
            (Lit::Double(lhs), TT::Greater, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs > rhs)),
            (Lit::Double(lhs), TT::GreaterEqual, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs >= rhs)),
            (Lit::Double(lhs), TT::Less, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs < rhs)),
            (Lit::Double(lhs), TT::LessEqual, Lit::Double(rhs)) => Ok(Lit::Boolean(lhs <= rhs)),
            (_, TT::Greater, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::GreaterEqual, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::Less, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::LessEqual, _) => Err("Operands must be numbers.".to_string()),
            (_, TT::EqualEqual, _) => Ok(Lit::Boolean(Interpreter::is_equal(&lval, &rval))),
            (_, TT::BangEqual, _) => Ok(Lit::Boolean(!Interpreter::is_equal(&lval, &rval))),
            _ => Ok(Lit::None),
        }
    }

    fn eval_grouping(&mut self, expr: &Expr) -> Result<Lit, String> {
        return self.evaluate(expr);
    }

    fn eval_literal(&mut self, lit: &Lit) -> Result<Lit, String> {
        return Ok(lit.clone());
    }

    fn eval_unary(&mut self, op: &Token, expr: &Expr) -> Result<Lit, String> {
        let lit = self.evaluate(expr)?;
        match (op.token, &lit) {
            (TT::Minus, Lit::Double(n)) => Ok(Lit::Double(-n)),
            (TT::Minus, _) => Err("Operand must be a number.".to_string()),
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
