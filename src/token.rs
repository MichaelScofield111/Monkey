#[derive(Debug, Clone, Default)]
pub struct Token {
    pub r#type: TokenType,
    #[allow(unused)]
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TokenType {
    Illegal,
    #[default]
    Eof,

    Ident,
    Int,
    String,

    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    Comma,
    Semicolon,
    Colon,

    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    LBracket,
    RBracket,

    Let,
    Function,
    If,
    Else,
    Return,
    True,
    False,

    Lt,
    Gt,

    Eq,
    Noteq,
}

impl Token {
    #[allow(dead_code)]
    pub fn new(r#type: TokenType, literal: String) -> Self {
        Token { r#type, literal }
    }

    pub fn lookup_ident(ident: String) -> TokenType {
        match ident.as_str() {
            "let" => TokenType::Let,
            "fn" => TokenType::Function,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "return" => TokenType::Return,
            "true" => TokenType::True,
            "false" => TokenType::False,
            _ => TokenType::Ident,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_next_token_simple() {
        let input = "=+(){},;";

        let tests = vec![
            (TokenType::Assign, "="),
            (TokenType::Plus, "+"),
            (TokenType::Lparen, "("),
            (TokenType::Rparen, ")"),
            (TokenType::Lbrace, "{"),
            (TokenType::Rbrace, "}"),
            (TokenType::Comma, ","),
            (TokenType::Semicolon, ";"),
            (TokenType::Eof, ""),
        ];

        let mut l = Lexer::new(input);

        for (i, (expected_type, expected_literal)) in tests.iter().enumerate() {
            let tok = l.next_token();

            assert_eq!(
                tok.r#type, *expected_type,
                "tests[{}] - tokentype wrong. expected={:?}, got={:?}",
                i, expected_type, tok.r#type
            );

            assert_eq!(
                tok.literal, *expected_literal,
                "tests[{}] - literal wrong. expected={}, got={}",
                i, expected_literal, tok.literal
            );
        }
    }

    #[test]
    fn test_next_token_snippet() {
        let input = r#"
               let five = 5;
               let ten = 10;

               let add = fn(x, y) {
	x + y;
               };

               let result = add(five, ten);

               !-/*5;
               5 < 10 > 5

               if (5 < 10) {
	return true;
               } else {
	return false;
               }

               10 == 10;
               10 != 9;
               "foobar"
               "foo bar"
               [
               ]
               {"foo": "bar"}
               "#;

        let tests = vec![
            (TokenType::Let, "let"),
            (TokenType::Ident, "five"),
            (TokenType::Assign, "="),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "ten"),
            (TokenType::Assign, "="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "add"),
            (TokenType::Assign, "="),
            (TokenType::Function, "fn"),
            (TokenType::Lparen, "("),
            (TokenType::Ident, "x"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "y"),
            (TokenType::Rparen, ")"),
            (TokenType::Lbrace, "{"),
            (TokenType::Ident, "x"),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "y"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "result"),
            (TokenType::Assign, "="),
            (TokenType::Ident, "add"),
            (TokenType::Lparen, "("),
            (TokenType::Ident, "five"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "ten"),
            (TokenType::Rparen, ")"),
            (TokenType::Semicolon, ";"),
            (TokenType::Bang, "!"),
            (TokenType::Minus, "-"),
            (TokenType::Slash, "/"),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Gt, ">"),
            (TokenType::Int, "5"),
            (TokenType::If, "if"),
            (TokenType::Lparen, "("),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Rparen, ")"),
            (TokenType::Lbrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::True, "true"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Else, "else"),
            (TokenType::Lbrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::Rbrace, "}"),
            (TokenType::Int, "10"),
            (TokenType::Eq, "=="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "10"),
            (TokenType::Noteq, "!="),
            (TokenType::Int, "9"),
            (TokenType::Semicolon, ";"),
            (TokenType::String, "foobar"),
            (TokenType::String, "foo bar"),
            (TokenType::LBracket, "["),
            (TokenType::RBracket, "]"),
            (TokenType::Lbrace, "{"),
            (TokenType::String, "foo"),
            (TokenType::Colon, ":"),
            (TokenType::String, "bar"),
            (TokenType::Rbrace, "}"),
            (TokenType::Eof, ""),
        ];

        let mut l = Lexer::new(input);

        for (i, (expected_type, expected_literal)) in tests.iter().enumerate() {
            let tok = l.next_token();

            assert_eq!(
                tok.r#type, *expected_type,
                "tests[{}] - tokentype wrong. expected={:?}, got={:?}",
                i, expected_type, tok.r#type
            );

            assert_eq!(
                tok.literal, *expected_literal,
                "tests[{}] - literal wrong. expected={}, got={}",
                i, expected_literal, tok.literal
            );
        }
    }
}
