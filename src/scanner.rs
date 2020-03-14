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
            Some(x) if *x == expected => {
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
            Some('(') => self.add_token(TokenType::LeftParen, c.unwrap().to_string()),
            Some(')') => self.add_token(TokenType::RightParen, c.unwrap().to_string()),
            Some('{') => self.add_token(TokenType::LeftBrace, c.unwrap().to_string()),
            Some('}') => self.add_token(TokenType::RightBrace, c.unwrap().to_string()),
            Some(',') => self.add_token(TokenType::Comma, c.unwrap().to_string()),
            Some('.') => self.add_token(TokenType::Dot, c.unwrap().to_string()),
            Some('-') => self.add_token(TokenType::Minus, c.unwrap().to_string()),
            Some('+') => self.add_token(TokenType::Plus, c.unwrap().to_string()),
            Some(';') => self.add_token(TokenType::Semicolon, c.unwrap().to_string()),
            Some('*') => self.add_token(TokenType::Star, c.unwrap().to_string()),

            Some('!') if self.munch('=') => self.add_token(TokenType::BangEqual, "!=".to_string()),
            Some('!') => self.add_token(TokenType::Bang, c.unwrap().to_string()),
            Some('=') if self.munch('=') => self.add_token(TokenType::EqualEqual, "==".to_string()),
            Some('=') => self.add_token(TokenType::Equal, c.unwrap().to_string()),
            Some('<') if self.munch('=') => self.add_token(TokenType::LessEqual, "<=".to_string()),
            Some('<') => self.add_token(TokenType::Less, c.unwrap().to_string()),
            Some('>') if self.munch('=') => self.add_token(TokenType::GreaterEqual, ">=".to_string()),
            Some('>') => self.add_token(TokenType::Greater, c.unwrap().to_string()),

            Some('/') =>
                if self.munch('/') {
                    self.chars.by_ref().take_while(|x| *x != '\n');
                    self.line += 1;
                    self.current = 0;
                } else {
                    self.add_token(TokenType::Slash, c.unwrap().to_string());
                },

            Some(' ') | Some('\t') | Some('\r') => (),
            Some('\n') => {
                self.line += 1;
                self.current = 0;
            }

            _ => (),
        }
    }

    fn add_token(&mut self, token: TokenType, lexeme: String) {
        self.tokens.push(Token{token, lexeme, line: self.line});
    }
}
