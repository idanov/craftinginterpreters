use itertools::peek_nth;

use itertools::structs::PeekNth;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::str::Chars;

use crate::lox_callable::{LoxCallable, LoxInstance};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Double(f64),
    String(String),
    Boolean(bool),
    Callable(LoxCallable),
    LoxInstance(Rc<RefCell<LoxInstance>>),
    None,
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Double(a), Literal::Double(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            (Literal::Callable(a), Literal::Callable(b)) => a == b,
            (Literal::LoxInstance(a), Literal::LoxInstance(b)) => Rc::ptr_eq(a, b),
            (Literal::None, Literal::None) => true,
            _ => false,
        }
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::Double(float) => float.to_bits().hash(state),
            Literal::String(string) => string.hash(state),
            Literal::Boolean(boolean) => boolean.hash(state),
            Literal::Callable(callable) => callable.hash(state),
            Literal::LoxInstance(instance) => Rc::as_ptr(instance).hash(state),
            Literal::None => 0.hash(state),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Double(num) if num.fract() == 0.0 => write!(f, "{}", *num as i64),
            Literal::Double(num) => write!(f, "{}", num),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Callable(lox) => write!(f, "{}", lox),
            Literal::LoxInstance(lox) => write!(f, "{}", lox.borrow()),
            Literal::None => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

pub struct Scanner<'a> {
    chars: PeekNth<Chars<'a>>,
    tokens: Vec<Result<Token, String>>,
    current: usize,
    line: usize,
    keywords: HashMap<&'a str, TokenType>,
}

impl Scanner<'_> {
    pub fn new(source: &str) -> Scanner {
        let keywords: HashMap<&str, TokenType> = [
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
        .iter()
        .cloned()
        .collect();

        Scanner {
            chars: peek_nth(source.chars()),
            tokens: Vec::new(),
            current: 0,
            line: 1,
            keywords,
        }
    }

    fn munch(&mut self, expected: char) -> bool {
        let res = self.chars.next_if_eq(&expected).is_some();
        self.current += res as usize;
        res
    }

    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&'\0')
    }

    fn peek_next(&mut self) -> char {
        *self.chars.peek_nth(1).unwrap_or(&'\0')
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }

    pub fn scan_tokens(&mut self) -> &[Result<Token, String>] {
        while self.chars.peek().is_some() {
            self.scan_token();
        }

        self.add_token(TokenType::Eof, "".into());
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            Some(x @ '(') => self.add_token(TokenType::LeftParen, x.into()),
            Some(x @ ')') => self.add_token(TokenType::RightParen, x.into()),
            Some(x @ '{') => self.add_token(TokenType::LeftBrace, x.into()),
            Some(x @ '}') => self.add_token(TokenType::RightBrace, x.into()),
            Some(x @ ',') => self.add_token(TokenType::Comma, x.into()),
            Some(x @ '.') => self.add_token(TokenType::Dot, x.into()),
            Some(x @ '-') => self.add_token(TokenType::Minus, x.into()),
            Some(x @ '+') => self.add_token(TokenType::Plus, x.into()),
            Some(x @ ';') => self.add_token(TokenType::Semicolon, x.into()),
            Some(x @ '*') => self.add_token(TokenType::Star, x.into()),

            Some('!') if self.munch('=') => self.add_munched_token(TokenType::BangEqual, "!=".into()),
            Some(x @ '!') => self.add_token(TokenType::Bang, x.into()),
            Some('=') if self.munch('=') => self.add_munched_token(TokenType::EqualEqual, "==".into()),
            Some(x @ '=') => self.add_token(TokenType::Equal, x.into()),
            Some('<') if self.munch('=') => self.add_munched_token(TokenType::LessEqual, "<=".into()),
            Some(x @ '<') => self.add_token(TokenType::Less, x.into()),
            Some('>') if self.munch('=') => self.add_munched_token(TokenType::GreaterEqual, ">=".into()),
            Some(x @ '>') => self.add_token(TokenType::Greater, x.into()),

            Some('/') if self.munch('/') => {
                let _: String = self.chars.by_ref().take_while(|&x| x != '\n').collect();
                self.line += 1;
                self.current = 0;
            }
            Some(x @ '/') => self.add_token(TokenType::Slash, x.into()),
            Some(' ') | Some('\t') | Some('\r') => (),
            Some('\n') => {
                self.line += 1;
                self.current = 0;
            }

            Some('"') => {
                let mut lines = 0;
                let mut count = self.current;
                let res: String = self
                    .chars
                    .take_while_ref(|&x| match x {
                        '"' => false,
                        '\n' => {
                            lines += 1;
                            count = 0;
                            true
                        }
                        _ => {
                            count += 1;
                            true
                        }
                    })
                    .collect();
                if self.chars.peek().is_none() {
                    self.tokens.push(Err(format!(
                        "[line {}:{}] Error: Unterminated string.",
                        self.line, self.current
                    )))
                } else {
                    self.add_string_token(TokenType::String, &res);
                    self.line += lines;
                    self.current = count;
                    self.advance(); // consume final "
                }
            }

            Some(x) if x.is_ascii_digit() => {
                let mut digits: String = x.to_string();
                digits.extend(self.chars.take_while_ref(|y| y.is_ascii_digit()));
                if self.peek() == '.' && self.peek_next().is_ascii_digit() {
                    digits.extend(self.chars.next());
                    digits.extend(self.chars.take_while_ref(|y| y.is_ascii_digit()));
                }
                let count = digits.len() - 1;
                self.add_numeric_token(TokenType::Number, digits);
                self.current += count;
            }
            Some(x) if x.is_alphabetic() || x == '_' => {
                let mut ident: String = x.to_string();
                ident.extend(
                    self.chars
                        .take_while_ref(|y| y.is_alphanumeric() || *y == '_'),
                );
                let count = ident.len() - 1;
                let token = self.keywords.get(ident.as_str()).copied();
                match token {
                    Some(y) => self.add_token(y, ident),
                    None => self.add_token(TokenType::Identifier, ident),
                }
                self.current += count;
            }

            _ => self.tokens.push(Err(format!(
                "[line {}:{}] Error: Unexpected character.",
                self.line, self.current
            ))),
        }
    }

    fn add_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Ok(Token {
            token,
            lexeme,
            literal: Literal::None,
            line: self.line,
            column: self.current,
        }));
    }

    fn add_munched_token(&mut self, token: TokenType, lexeme: String) {
        let offset = lexeme.len() - 1;
        self.tokens.push(Ok(Token {
            token,
            lexeme,
            literal: Literal::None,
            line: self.line,
            column: self.current - offset,
        }));
    }

    fn add_numeric_token(&mut self, token: TokenType, lexeme: String) {
        let num = lexeme.parse::<f64>().unwrap_or(0.0);
        self.tokens.push(Ok(Token {
            token,
            lexeme,
            literal: Literal::Double(num),
            line: self.line,
            column: self.current,
        }));
    }

    fn add_string_token(&mut self, token: TokenType, lexeme: &str) {
        self.tokens.push(Ok(Token {
            token,
            lexeme: lexeme.into(),
            literal: Literal::String(lexeme.into()),
            line: self.line,
            column: self.current,
        }));
    }
}
