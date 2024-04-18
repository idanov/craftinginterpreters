use crate::expr::Expr;
use crate::scanner::{Literal, Token, TokenType};
use crate::stmt::Stmt;
use colored::Colorize;
use itertools::peek_nth;
use itertools::structs::PeekNth;
use std::vec::IntoIter;

pub struct Parser {
    tokens: PeekNth<IntoIter<Token>>,
    prev: Option<Token>,
}

/****************************************************************
Parser grammar:

    program        → declaration* EOF ;

    declaration    → funDecl
                   | varDecl
                   | statement ;

    funDecl        → "fun" function ;
    function       → IDENTIFIER "(" parameters? ")" block ;
    parameters     → IDENTIFIER ( "," IDENTIFIER )* ;

    varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;

    statement      → exprStmt
                   | forStmt
                   | ifStmt
                   | printStmt
                   | returnStmt
                   | whileStmt
                   | block ;

    returnStmt     → "return" expression? ";" ;

    forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                   expression? ";"
                   expression? ")" statement ;

    whileStmt      → "while" "(" expression ")" statement ;

    ifStmt         → "if" "(" expression ")" statement
                   ( "else" statement )? ;

    block          → "{" declaration* "}" ;

    exprStmt       → expression ";" ;
    printStmt      → "print" expression ";" ;

    expression     → assignment ;
    assignment     → IDENTIFIER "=" assignment
                   | logic_or ;
    logic_or       → logic_and ( "or" logic_and )* ;
    logic_and      → equality ( "and" equality )* ;
    equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term           → factor ( ( "-" | "+" ) factor )* ;
    factor         → unary ( ( "/" | "*" ) unary )* ;
    unary          → ( "!" | "-" ) unary | call ;
    call           → primary ( "(" arguments? ")" )* ;
    arguments      → expression ( "," expression )* ;

    primary        → "true" | "false" | "nil"
                   | NUMBER | STRING
                   | "(" expression ")"
                   | IDENTIFIER ;

*****************************************************************/
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: peek_nth(tokens.into_iter()),
            prev: None,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let stmt = self.declaration();
            println!("{}", format!("Debug {:?}", stmt).dimmed());
            if let Ok(x) = stmt {
                statements.push(x);
            } else {
                self.synchronize();
            }
        }

        return Ok(statements);
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.munch(&[TokenType::Fun]) {
            return self.function("function");
        }
        if self.munch(&[TokenType::Var]) {
            return self.var_declaration();
        }
        return self.statement();
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, String> {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expect {} name.", kind).as_str(),
        )?;
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;

        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Parser::error::<Stmt>(
                        self.peek(),
                        "Can't have more than 255 parameters.".to_string(),
                    );
                }
                let param = self.consume(TokenType::Identifier, "Expect parameter name.")?;
                parameters.push(param);
                if !self.munch(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;

        let body = self.block()?;
        return Ok(Stmt::Function(name, parameters, body));
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer: Option<Expr> = if self.munch(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        return Ok(Stmt::Var(name, initializer));
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.munch(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.munch(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.munch(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.munch(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.munch(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.munch(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        return self.expression_statement();
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.munch(&[TokenType::Semicolon]) {
            None
        } else if self.munch(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let cond = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal::Boolean(true))
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        // Desugaring a for loop into a while loop
        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)])
        }
        body = Stmt::While(cond, Box::new(body));
        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body])
        }

        return Ok(body);
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch: Option<Box<Stmt>> = if self.munch(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        return Ok(Stmt::If(cond, then_branch, else_branch));
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        return Ok(Stmt::Print(value));
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.previous();
        let mut value = Expr::Literal(Literal::None);
        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        return Ok(Stmt::Return(keyword, value));
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        return Ok(Stmt::While(cond, Box::new(body)));
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        return Ok(Stmt::Expression(expr));
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        return Ok(statements);
    }

    fn expression(&mut self) -> Result<Expr, String> {
        return self.assignment();
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;
        if self.munch(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, Box::new(value)));
            }

            return Parser::error::<Expr>(equals, "Invalid assignment target.".to_string());
        }
        return Ok(expr);
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.and()?;
        while self.munch(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.equality()?;
        while self.munch(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        return Ok(expr);
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
        return self.call_expr();
    }

    fn call_expr(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.primary()?;

        loop {
            if self.munch(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Parser::error::<Expr>(
                        self.peek(),
                        "Can't have more than 255 arguments.".to_string(),
                    );
                }
                arguments.push(self.expression()?);
                if !self.munch(&[TokenType::Comma]) {
                    break;
                }
            }
        };

        let paren: Token = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        return Ok(Expr::Call(Box::new(callee), paren, arguments));
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

        if self.munch(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        }

        if self.munch(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        return Parser::error::<Expr>(self.peek(), "Expect expression.".to_string());
    }

    fn consume(&mut self, types: TokenType, message: &str) -> Result<Token, String> {
        if self.check(types) {
            return Ok(self.advance());
        }
        let prev = self.previous();
        let msg = format!("{}. Last valid lexeme was {}.", message, prev.lexeme);
        return Parser::error::<Token>(self.peek(), msg);
    }

    pub fn error<T>(token: Token, message: String) -> Result<T, String> {
        if token.token == TokenType::EOF {
            return Err(format!(
                "[line {}:{}] Error at end: {}",
                token.line, token.column, message
            ));
        } else {
            return Err(format!(
                "[line {}:{}] Error at {}: {}",
                token.line, token.column, token.lexeme, message
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
            if self.check(*token) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        };
        return self.peek().token == token;
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
