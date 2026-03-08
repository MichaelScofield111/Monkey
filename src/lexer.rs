use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize, // current char

    // TODO: unicode and utf-8 support
    ch: char, // the char being read (at position)

    read_position: usize, // the char after current char
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            input: input.chars().collect(),
            position: 0,
            ch: '\0',
            read_position: 0,
        };

        lexer.read_char(); // 👈 关键！初始化 ch 为第一个字符
        lexer
    }

    pub fn read_identifier(&mut self) -> String {
        let position = self.position;
        while Lexer::is_letter(self.ch) {
            self.read_char();
        }
        self.input[position..self.position]
            .iter()
            .collect::<String>()
    }

    pub fn read_digit(&mut self) -> String {
        let position = self.position;
        while Lexer::is_digit(self.ch) {
            self.read_char();
        }
        self.input[position..self.position]
            .iter()
            .collect::<String>()
    }
    pub fn is_letter(ch: char) -> bool {
        return 'a' <= ch && ch <= 'z' || 'A' <= ch && ch <= 'Z' || ch == '_';
    }

    pub fn is_digit(ch: char) -> bool {
        ('0'..='9').contains(&ch)
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_wite_spaces();

        let tok = match self.ch {
            '=' => Token {
                r#type: TokenType::ASSIGN,
                literal: self.ch.to_string(),
            },
            ';' => Token {
                r#type: TokenType::SEMICOLON,
                literal: self.ch.to_string(),
            },
            '(' => Token {
                r#type: TokenType::LPAREN,
                literal: self.ch.to_string(),
            },
            ')' => Token {
                r#type: TokenType::RPAREN,
                literal: self.ch.to_string(),
            },
            '{' => Token {
                r#type: TokenType::LBRACE,
                literal: self.ch.to_string(),
            },
            '}' => Token {
                r#type: TokenType::RBRACE,
                literal: self.ch.to_string(),
            },
            ',' => Token {
                r#type: TokenType::COMMA,
                literal: self.ch.to_string(),
            },
            '+' => Token {
                r#type: TokenType::PLUS,
                literal: self.ch.to_string(),
            },
            '\0' => Token {
                r#type: TokenType::EOF,
                literal: "".to_string(),
            },
            _ => {
                if Lexer::is_letter(self.ch) {
                    let literal = self.read_identifier();
                    let r#type = Token::lookup_ident(literal.clone());
                    return Token { r#type, literal };
                } else if Lexer::is_digit(self.ch) {
                    let literal = self.read_digit();
                    return Token {
                        r#type: TokenType::INT,
                        literal,
                    };
                } else {
                    Token {
                        r#type: TokenType::ILLEGAL,
                        literal: self.ch.to_string(),
                    }
                }
            }
        };

        self.read_char();
        tok
    }

    fn skip_wite_spaces(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }
}
