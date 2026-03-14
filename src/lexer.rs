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
                        r#type: TokenType::Eq,
                        literal: liter,
                    }
                } else {
                    Token {
                        r#type: TokenType::Assign,
                        literal: (self.ch as char).to_string(),
                    }
                }
            }
            b'-' => Token {
                r#type: TokenType::Minus,
                literal: (self.ch as char).to_string(),
            },
            b'*' => Token {
                r#type: TokenType::Asterisk,
                literal: (self.ch as char).to_string(),
            },
            b'/' => Token {
                r#type: TokenType::Slash,
                literal: (self.ch as char).to_string(),
            },
            b';' => Token {
                r#type: TokenType::Semicolon,
                literal: (self.ch as char).to_string(),
            },
            b'(' => Token {
                r#type: TokenType::Lparen,
                literal: (self.ch as char).to_string(),
            },
            b')' => Token {
                r#type: TokenType::Rparen,
                literal: (self.ch as char).to_string(),
            },
            b'{' => Token {
                r#type: TokenType::Lbrace,
                literal: (self.ch as char).to_string(),
            },
            b'}' => Token {
                r#type: TokenType::Rbrace,
                literal: (self.ch as char).to_string(),
            },
            b',' => Token {
                r#type: TokenType::Comma,
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
                        r#type: TokenType::Noteq,
                        literal: liter,
                    }
                } else {
                    Token {
                        r#type: TokenType::Bang,
                        literal: (self.ch as char).to_string(),
                    }
                }
            }
            b'+' => Token {
                r#type: TokenType::Plus,
                literal: (self.ch as char).to_string(),
            },
            b'<' => Token {
                r#type: TokenType::Lt,
                literal: (self.ch as char).to_string(),
            },
            b'>' => Token {
                r#type: TokenType::Gt,
                literal: (self.ch as char).to_string(),
            },
            0 => Token {
                r#type: TokenType::Eof,
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
                        r#type: TokenType::Int,
                        literal,
                    };
                } else {
                    Token {
                        r#type: TokenType::Illegal,
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
