use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    position: usize, // current char

    // TODO: unicode and utf-8 support
    ch: u8, // the char being read (at position)

    read_position: usize, // the char after current char
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input: input.as_bytes(),
            position: 0,
            ch: 0,
            read_position: 0,
        };

        lexer.read_char();
        lexer
    }

    pub fn read_identifier(&mut self) -> String {
        let position = self.position;
        while Self::is_letter(self.ch) {
            self.read_char();
        }

        std::str::from_utf8(&self.input[position..self.position])
            .unwrap()
            .to_string()
    }

    pub fn read_digit(&mut self) -> String {
        let position = self.position;
        while Self::is_digit(self.ch) {
            self.read_char();
        }
        std::str::from_utf8(&self.input[position..self.position])
            .unwrap()
            .to_string()
    }
    pub fn is_letter(ch: u8) -> bool {
        ch.is_ascii_alphabetic() || ch == b'_'
    }

    pub fn is_digit(ch: u8) -> bool {
        ch.is_ascii_digit()
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespaces();

        let tok = match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    // 10 == 10
                    let ch = (self.ch as char).to_string();
                    self.read_char();
                    let n_ch = (self.ch as char).to_string();
                    let liter = ch + &n_ch;
                    Token {
                        r#type: TokenType::EQ,
                        literal: liter,
                    }
                } else {
                    Token {
                        r#type: TokenType::ASSIGN,
                        literal: (self.ch as char).to_string(),
                    }
                }
            }
            b'-' => Token {
                r#type: TokenType::MINUS,
                literal: (self.ch as char).to_string(),
            },
            b'*' => Token {
                r#type: TokenType::ASTERISK,
                literal: (self.ch as char).to_string(),
            },
            b'/' => Token {
                r#type: TokenType::SLASH,
                literal: (self.ch as char).to_string(),
            },
            b';' => Token {
                r#type: TokenType::SEMICOLON,
                literal: (self.ch as char).to_string(),
            },
            b'(' => Token {
                r#type: TokenType::LPAREN,
                literal: (self.ch as char).to_string(),
            },
            b')' => Token {
                r#type: TokenType::RPAREN,
                literal: (self.ch as char).to_string(),
            },
            b'{' => Token {
                r#type: TokenType::LBRACE,
                literal: (self.ch as char).to_string(),
            },
            b'}' => Token {
                r#type: TokenType::RBRACE,
                literal: (self.ch as char).to_string(),
            },
            b',' => Token {
                r#type: TokenType::COMMA,
                literal: (self.ch as char).to_string(),
            },
            b'!' => {
                if self.peek_char() == b'=' {
                    // 10 != 5
                    let ch = (self.ch as char).to_string();
                    self.read_char();
                    let n_ch = (self.ch as char).to_string();
                    let liter = ch + &n_ch;
                    Token {
                        r#type: TokenType::NOTEQ,
                        literal: liter,
                    }
                } else {
                    Token {
                        r#type: TokenType::BANG,
                        literal: (self.ch as char).to_string(),
                    }
                }
            }
            b'+' => Token {
                r#type: TokenType::PLUS,
                literal: (self.ch as char).to_string(),
            },
            b'<' => Token {
                r#type: TokenType::LT,
                literal: (self.ch as char).to_string(),
            },
            b'>' => Token {
                r#type: TokenType::GT,
                literal: (self.ch as char).to_string(),
            },
            0 => Token {
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

    fn skip_whitespaces(&mut self) {
        while matches!(self.ch, b' ' | b'\t' | b'\n' | b'\r') {
            self.read_char();
        }
    }

    fn peek_char(&self) -> u8 {
        if self.read_position >= self.input.len() {
            0
        } else {
            self.input[self.read_position]
        }
    }
}
