#[derive(Debug, Clone, Default)]
pub struct Token {
    pub r#type: TokenType,
    #[allow(unused)]
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TokenType {
    ILLEGAL,
    #[default]
    EOF,

    IDENT,
    INT,

    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,

    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    LET,
    FUNCTION,
    IF,
    ELSE,
    RETURN,
    TRUE,
    FALSE,

    LT,
    GT,

    EQ,
    NOTEQ,
}

impl Token {
    #[allow(dead_code)]
    pub fn new(r#type: TokenType, literal: String) -> Self {
        Token { r#type, literal }
    }

    pub fn lookup_ident(ident: String) -> TokenType {
        match ident.as_str() {
            "let" => TokenType::LET,
            "fn" => TokenType::FUNCTION,
            "if" => TokenType::IF,
            "else" => TokenType::ELSE,
            "return" => TokenType::RETURN,
            "true" => TokenType::TRUE,
            "false" => TokenType::FALSE,
            _ => TokenType::IDENT,
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
            (TokenType::ASSIGN, "="),
            (TokenType::PLUS, "+"),
            (TokenType::LPAREN, "("),
            (TokenType::RPAREN, ")"),
            (TokenType::LBRACE, "{"),
            (TokenType::RBRACE, "}"),
            (TokenType::COMMA, ","),
            (TokenType::SEMICOLON, ";"),
            (TokenType::EOF, ""),
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
               "#;

        let tests = vec![
            (TokenType::LET, "let"),
            (TokenType::IDENT, "five"),
            (TokenType::ASSIGN, "="),
            (TokenType::INT, "5"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::LET, "let"),
            (TokenType::IDENT, "ten"),
            (TokenType::ASSIGN, "="),
            (TokenType::INT, "10"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::LET, "let"),
            (TokenType::IDENT, "add"),
            (TokenType::ASSIGN, "="),
            (TokenType::FUNCTION, "fn"),
            (TokenType::LPAREN, "("),
            (TokenType::IDENT, "x"),
            (TokenType::COMMA, ","),
            (TokenType::IDENT, "y"),
            (TokenType::RPAREN, ")"),
            (TokenType::LBRACE, "{"),
            (TokenType::IDENT, "x"),
            (TokenType::PLUS, "+"),
            (TokenType::IDENT, "y"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::RBRACE, "}"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::LET, "let"),
            (TokenType::IDENT, "result"),
            (TokenType::ASSIGN, "="),
            (TokenType::IDENT, "add"),
            (TokenType::LPAREN, "("),
            (TokenType::IDENT, "five"),
            (TokenType::COMMA, ","),
            (TokenType::IDENT, "ten"),
            (TokenType::RPAREN, ")"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::BANG, "!"),
            (TokenType::MINUS, "-"),
            (TokenType::SLASH, "/"),
            (TokenType::ASTERISK, "*"),
            (TokenType::INT, "5"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::INT, "5"),
            (TokenType::LT, "<"),
            (TokenType::INT, "10"),
            (TokenType::GT, ">"),
            (TokenType::INT, "5"),
            (TokenType::IF, "if"),
            (TokenType::LPAREN, "("),
            (TokenType::INT, "5"),
            (TokenType::LT, "<"),
            (TokenType::INT, "10"),
            (TokenType::RPAREN, ")"),
            (TokenType::LBRACE, "{"),
            (TokenType::RETURN, "return"),
            (TokenType::TRUE, "true"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::RBRACE, "}"),
            (TokenType::ELSE, "else"),
            (TokenType::LBRACE, "{"),
            (TokenType::RETURN, "return"),
            (TokenType::FALSE, "false"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::RBRACE, "}"),
            (TokenType::INT, "10"),
            (TokenType::EQ, "=="),
            (TokenType::INT, "10"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::INT, "10"),
            (TokenType::NOTEQ, "!="),
            (TokenType::INT, "9"),
            (TokenType::SEMICOLON, ";"),
            (TokenType::EOF, ""),
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
