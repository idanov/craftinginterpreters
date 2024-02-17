use std::str::Chars;
use itertools::Itertools;
use itertools::peek_nth;
use std::collections::HashMap;
use itertools::structs::PeekNth;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

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
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EOF
}

#[derive(Debug, Clone)]
pub enum Literal {
    Double(f64),
    String(String),
    Boolean(bool),
    None
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
    pub column: usize
}

pub struct Scanner<'a> {
    chars: PeekNth<Chars<'a>>,
    tokens: Vec<Result<Token, String>>,
    current: usize,
    line: usize,
    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &String) -> Scanner {
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
        ].iter().cloned().collect();

        Scanner {
            chars: peek_nth(source.chars()),
            tokens: Vec::new(),
            current: 0,
            line: 1,
            keywords
        }
    }

    fn munch(&mut self, expected: char) -> bool {
        let res = self.chars.next_if_eq(&expected).is_some();
        self.current += res as usize;
        return res;
    }

    fn peek(&mut self) -> char {
        return *self.chars.peek().unwrap_or(&'\0')
    }

    fn peek_next(&mut self) -> char {
        return *self.chars.peek_nth(1).unwrap_or(&'\0')
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        return self.chars.next()
    }

    pub fn scan_tokens(&mut self) -> &Vec<Result<Token, String>> {
        while self.chars.peek().is_some() {
            self.scan_token();
        }

        self.add_token(TokenType::EOF, String::from(""));
        return &self.tokens;
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            Some(x @ '(') => self.add_token(TokenType::LeftParen, x.to_string()),
            Some(x @ ')') => self.add_token(TokenType::RightParen, x.to_string()),
            Some(x @ '{') => self.add_token(TokenType::LeftBrace, x.to_string()),
            Some(x @ '}') => self.add_token(TokenType::RightBrace, x.to_string()),
            Some(x @ ',') => self.add_token(TokenType::Comma, x.to_string()),
            Some(x @ '.') => self.add_token(TokenType::Dot, x.to_string()),
            Some(x @ '-') => self.add_token(TokenType::Minus, x.to_string()),
            Some(x @ '+') => self.add_token(TokenType::Plus, x.to_string()),
            Some(x @ ';') => self.add_token(TokenType::Semicolon, x.to_string()),
            Some(x @ '*') => self.add_token(TokenType::Star, x.to_string()),

            Some('!') if self.munch('=') => self.add_token(TokenType::BangEqual, "!=".to_string()),
            Some(x @ '!') => self.add_token(TokenType::Bang, x.to_string()),
            Some('=') if self.munch('=') => self.add_token(TokenType::EqualEqual, "==".to_string()),
            Some(x @ '=') => self.add_token(TokenType::Equal, x.to_string()),
            Some('<') if self.munch('=') => self.add_token(TokenType::LessEqual, "<=".to_string()),
            Some(x @ '<') => self.add_token(TokenType::Less, x.to_string()),
            Some('>') if self.munch('=') => self.add_token(TokenType::GreaterEqual, ">=".to_string()),
            Some(x @ '>') => self.add_token(TokenType::Greater, x.to_string()),

            Some('/') if self.munch('/') => {
                let _: String = self.chars.by_ref().take_while(|&x| x != '\n').collect();
                self.line += 1;
                self.current = 0;
            },
            Some(x @ '/') => self.add_token(TokenType::Slash, x.to_string()),
            Some(' ') | Some('\t') | Some('\r') => (),
            Some('\n') => {
                self.line += 1;
                self.current = 0;
            },

            Some('"') => {
                let mut lines = 0;
                let mut count = self.current;
                let res: String = self.chars.take_while_ref(|&x| {
                    count += 1;
                    if x == '\n' {
                        lines += 1;
                        count = 0;
                    };
                    x != '"'
                }).collect();
                if self.chars.peek().is_none() {
                    self.tokens.push(Err(format!("Error: Unterminated string at line {:?} and column {:?}.", self.line, self.current)))
                } else {
                    self.add_string_token(TokenType::String, res);
                    self.line += lines;
                    self.current = count;
                    self.advance(); // consume final "
                }
            },

            Some(x) if x.is_ascii_digit() => {
                let mut digits:String = String::from(x.to_string());
                digits.extend(self.chars.take_while_ref(|y| y.is_ascii_digit()));
                if self.peek() == '.' && self.peek_next().is_ascii_digit() {
                    digits.extend(self.chars.next());
                    digits.extend(self.chars.take_while_ref(|y| y.is_ascii_digit()));
                }
                let count = digits.len() - 1;
                self.add_numeric_token(TokenType::Number, digits);
                self.current += count;
            },
            Some(x) if x.is_alphabetic() || x == '_' => {
                let mut ident:String = String::from(x.to_string());
                ident.extend(self.chars.take_while_ref(|y| y.is_alphanumeric() || *y == '_'));
                let count = ident.len() - 1;
                let token = self.keywords.get(ident.as_str()).map(|y| y.clone());
                match token {
                    Some(y) => self.add_token(y, String::from("")),
                    None => self.add_token(TokenType::Identifier, ident),
                }
                self.current += count;
            },

            _ => {
                self.tokens.push(Err(format!("Error: Unexpected character at line {:?} and column {:?}.", self.line, self.current)))
            },
        }
    }

    fn add_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Ok(Token{token, lexeme, literal: Literal::None, line: self.line, column: self.current }));
    }

    fn add_numeric_token(&mut self, token: TokenType, lexeme: String) {
        let num = lexeme.parse::<f64>().unwrap_or(0.0);
        self.tokens.push(Ok(Token { token, lexeme, literal: Literal::Double(num), line: self.line, column: self.current }));
    }

    fn add_string_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Ok(Token { token, lexeme: lexeme.clone(), literal: Literal::String(lexeme), line: self.line, column: self.current }));
    }
}
