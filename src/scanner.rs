use std::str::Chars;
use itertools::Itertools;
use std::collections::HashMap;
use std::iter::Peekable;

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub struct Token {
    token: TokenType,
    lexeme: String,
    line: usize,
}

pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
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
            chars: source.chars().peekable(),
            tokens: Vec::new(),
            current: 0,
            line: 1,
            keywords
        }
    }

    fn munch(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            Some(&x) if x == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }

    pub fn scan_tokens(&mut self) -> &Vec<Result<Token, String>> {
        while self.chars.peek() != None {
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
                let res: String = self.chars.by_ref().take_while(|&x| {
                    if x == '\n' {
                        lines += 1;
                    };
                    x != '"'
                }).collect();
                self.add_token(TokenType::String, res);
                self.line += lines;
                if self.chars.peek().is_none() {
                    self.tokens.push(Err(format!("Error: Unterminated string at line {:?}.", self.line)))
                }
            },

            Some(x) if x.is_numeric() => {
                let mut digits:String = String::from(x.to_string());
                digits.extend(self.chars.take_while_ref(|y| y.is_numeric()));
                let mut lookahead = self.chars.clone();
                if lookahead.next().filter(|y| *y == '.').is_some() && lookahead.next().filter(|y| y.is_numeric()).is_some() {
                    digits.extend(self.chars.next());
                    digits.extend(self.chars.take_while_ref(|y| y.is_numeric()));
                }
                self.add_token(TokenType::Number, digits);
            },
            Some(x) if x.is_alphabetic() => {
                let mut ident:String = String::from(x.to_string());
                ident.extend(self.chars.take_while_ref(|y| y.is_alphanumeric()));
                let token = self.keywords.get(ident.as_str()).map(|y| y.clone());
                match token {
                    Some(y) => self.add_token(y, String::from("")),
                    None => self.add_token(TokenType::Identifier, ident),
                }
            },

            _ => {
                self.tokens.push(Err(format!("Error: Unexpected character at line {:?}.", self.line)))
            },
        }
    }

    fn add_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Ok(Token{token, lexeme, line: self.line}));
    }
}
