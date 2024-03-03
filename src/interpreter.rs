use crate::expr::Expr;
use crate::scanner::{Token, Literal as Lit, TokenType as TT};

pub struct Interpreter {
}


impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Lit {
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

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Lit {
        let lval = self.evaluate(left);
        let rval = self.evaluate(right);
        match (&lval, op.token, &rval) {
            (Lit::Double(lhs), TT::Minus, Lit::Double(rhs)) => Lit::Double(lhs - rhs),
            (Lit::Double(lhs), TT::Slash, Lit::Double(rhs)) => Lit::Double(lhs / rhs),
            (Lit::Double(lhs), TT::Star, Lit::Double(rhs)) => Lit::Double(lhs * rhs),
            (Lit::Double(lhs), TT::Plus, Lit::Double(rhs)) => Lit::Double(lhs + rhs),
            (Lit::String(lhs), TT::Plus, Lit::String(rhs)) => Lit::String(lhs.to_string() + rhs),
            (Lit::Double(lhs), TT::Greater, Lit::Double(rhs)) => Lit::Boolean(lhs > rhs),
            (Lit::Double(lhs), TT::GreaterEqual, Lit::Double(rhs)) => Lit::Boolean(lhs >= rhs),
            (Lit::Double(lhs), TT::Less, Lit::Double(rhs)) => Lit::Boolean(lhs < rhs),
            (Lit::Double(lhs), TT::LessEqual, Lit::Double(rhs)) => Lit::Boolean(lhs <= rhs),
            (_, TT::EqualEqual, _) => Lit::Boolean(Interpreter::is_equal(&lval, &rval)),
            (_, TT::BangEqual, _) => Lit::Boolean(!Interpreter::is_equal(&lval, &rval)),
            _ => Lit::None,
        }
    }

    fn eval_grouping(&mut self, expr: &Expr) -> Lit {
        return self.evaluate(expr);
    }

    fn eval_literal(&mut self, lit: &Lit) -> Lit {
        return lit.clone();
    }

    fn eval_unary(&mut self, op: &Token, expr: &Expr) -> Lit {
        let lit = self.evaluate(expr);
        match (op.token, &lit) {
            (TT::Minus, Lit::Double(n)) => Lit::Double(-n),
            (TT::Bang, _) => Lit::Boolean(!Interpreter::is_truthy(&lit)),
            _ => Lit::None,
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
