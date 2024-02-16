use crate::expr::Expr;
use crate::scanner::{Token, TokenType, Literal};
use std::vec::IntoIter;
use itertools::peek_nth;
use itertools::structs::PeekNth;

pub struct Parser {
    tokens: PeekNth<IntoIter<Token>>,
}


impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: peek_nth(tokens.into_iter()),
        }
    }

    fn expression(&mut self) -> Expr {
       return self.equality();
    }

    fn equality(&mut self) -> Expr {
        let mut expr: Expr = self.comparison();

        while self.munch(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr: Expr = self.term();

        while self.munch(&[TokenType::Greater,
                           TokenType::GreaterEqual,
                           TokenType::Less,
                           TokenType::LessEqual
                          ]) {
            let operator: Token = self.previous();
            let right: Expr = self.term();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr: Expr = self.factor();

        while self.munch(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right: Expr = self.factor();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr: Expr = self.unary();

        while self.munch(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return expr;
    }

    fn unary(&mut self) -> Expr {
        if self.munch(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary();
            return Expr::Unary(operator, Box::new(right));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Expr {
        if self.munch(&[TokenType::False]) {
            return Expr::Literal(Literal::Boolean(false));
        }
        if self.munch(&[TokenType::True]) {
            return Expr::Literal(Literal::Boolean(true));
        }
        if self.munch(&[TokenType::False]) {
            return Expr::Literal(Literal::None);
        }

        if self.munch(&[TokenType::Number, TokenType::String]) {
            return Expr::Literal(self.previous().literal);
        }

        if self.munch(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping(Box::new(expr));
        }
        return Expr::Literal(Literal::None); // this one is temporary, but it should return error
    }

    fn consume(&mut self, types: TokenType, message: &str) {
        todo!();
    }

    fn munch(&mut self, types: &[TokenType]) -> bool {
        for token in types {
            if self.check(*token) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token: TokenType) -> bool {
        if self.is_at_end() { return false };
        return self.peek().token == token;
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            return self.tokens.next().expect("No more tokens to be processed");
        }
        return self.previous();
    }

    fn is_at_end(&mut self) -> bool {
        return self.peek().token == TokenType::EOF;
    }

    fn peek(&mut self) -> Token {
        return self.tokens.peek_nth(1).expect("No more tokens to be processed").clone();
    }

    fn previous(&mut self) -> Token {
        return self.tokens.peek_nth(0).expect("No more tokens to be processed").clone();
    }
}
