use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug)]
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
    tokens: Vec<Token>,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Scanner<'a> {
        let chars = source.chars().peekable();
        Scanner {
            chars,
            tokens: Vec::new(),
            current: 0,
            line: 1,
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

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while self.chars.peek() != None {
            self.scan_token();
        }

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

            Some(x @ '!') if self.munch('=') => self.add_token(TokenType::BangEqual, "!=".to_string()),
            Some(x @ '!') => self.add_token(TokenType::Bang, x.to_string()),
            Some(x @ '=') if self.munch('=') => self.add_token(TokenType::EqualEqual, "==".to_string()),
            Some(x @ '=') => self.add_token(TokenType::Equal, x.to_string()),
            Some(x @ '<') if self.munch('=') => self.add_token(TokenType::LessEqual, "<=".to_string()),
            Some(x @ '<') => self.add_token(TokenType::Less, x.to_string()),
            Some(x @ '>') if self.munch('=') => self.add_token(TokenType::GreaterEqual, ">=".to_string()),
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
            }

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
                // TODO: if self.chars.peek() is None (file ended), raise error for unterminated string
            },

            Some(x) if x.is_digit(10) => {
                // TODO: parse numbers
            },
            Some(x) if x.is_alphabetic() => {
                let mut ident:String = String::from(x.to_string());
                ident.push(x);
                while self.chars.peek().filter(|y| y.is_alphanumeric()).is_some() {
                    ident.push(self.chars.next().unwrap());
                };

                // TODO: Check for keywords or just make it an identifier
                self.add_token(TokenType::Identifier, ident);
            },

            _ => {
                // raise error for unexpected character
                ()
            },
        }
    }

    fn add_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Token{token, lexeme, line: self.line});
    }
}
