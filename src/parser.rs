use crate::expr::Expr;
use crate::scanner::{Literal, Token, TokenType};
use crate::stmt::Stmt;
use itertools::peek_nth;
use itertools::structs::PeekNth;
use std::vec::IntoIter;

pub struct Parser {
    tokens: PeekNth<IntoIter<Token>>,
    prev: Option<Token>,
}

/****************************************************************
Parser grammar:
    program        → statement* EOF ;

    statement      → exprStmt
                   | printStmt ;

    exprStmt       → expression ";" ;
    printStmt      → "print" expression ";" ;

    expression     → equality ;
    equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term           → factor ( ( "-" | "+" ) factor )* ;
    factor         → unary ( ( "/" | "*" ) unary )* ;
    unary          → ( "!" | "-" ) unary
                   | primary ;
    primary        → NUMBER | STRING | "true" | "false" | "nil"
                   | "(" expression ")" ;

*****************************************************************/
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: peek_nth(tokens.into_iter()),
            prev: None,
        }
    }

    pub fn parse(&mut self) -> Vec<Result<Stmt, String>> {
        let mut statements: Vec<Result<Stmt, String>> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement());
        }

        return statements;
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.munch(&[TokenType::Print]) {
            return self.print_statement();
        }

        return self.expression_statement();
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        return Ok(Stmt::Print(value));
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        return Ok(Stmt::Expression(expr));
    }

    fn expression(&mut self) -> Result<Expr, String> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.comparison()?;

        while self.munch(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.term()?;

        while self.munch(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator: Token = self.previous();
            let right: Expr = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.factor()?;

        while self.munch(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right: Expr = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.unary()?;

        while self.munch(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.munch(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }
        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.munch(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }
        if self.munch(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }
        if self.munch(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::None));
        }

        if self.munch(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal));
        }

        if self.munch(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        return Parser::error::<Expr>(self.peek(), "Expect expression.");
    }

    fn consume(&mut self, types: TokenType, message: &str) -> Result<Token, String> {
        if self.check(&types) {
            return Ok(self.advance());
        }
        return Parser::error::<Token>(self.peek(), message);
    }

    fn error<T>(token: Token, message: &str) -> Result<T, String> {
        if token.token == TokenType::EOF {
            return Err(format!(
                "[line {}:{}] Error at end: {}",
                token.line, token.column, message
            ));
        } else {
            return Err(format!(
                "[line {}:{}] Error at {:?}: {}",
                token.line, token.column, token, message
            ));
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token == TokenType::Semicolon {
                return;
            }

            if [
                TokenType::Class,
                TokenType::Fun,
                TokenType::Var,
                TokenType::For,
                TokenType::If,
                TokenType::While,
                TokenType::Print,
                TokenType::Return,
            ]
            .contains(&(self.peek().token))
            {
                return;
            }

            self.advance();
        }
    }

    fn munch(&mut self, types: &[TokenType]) -> bool {
        for token in types {
            if self.check(&token) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        };
        return self.peek().token == *token;
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.prev = self.tokens.next();
        }
        return self.previous();
    }

    fn is_at_end(&mut self) -> bool {
        return self.peek().token == TokenType::EOF;
    }

    fn peek(&mut self) -> Token {
        return self
            .tokens
            .peek()
            .expect("No more tokens to be processed")
            .clone();
    }

    fn previous(&mut self) -> Token {
        return self
            .prev
            .clone()
            .expect("No previous token to be processed")
            .clone();
    }
}
