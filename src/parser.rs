use crate::{
    ast::{Expression, Identifier, LetStatement, Program, Statement},
    lexer::Lexer,
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>, // parser has lexer ownership

    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            cur_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_programe(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token.r#type != TokenType::EOF {
            let stmt = self.parse_statement();

            if let Some(stmt) = stmt {
                program.statements.push(stmt);
            }

            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.r#type {
            TokenType::LET => self.parse_let_statement(),
            _ => {
                println!("unsupported type: {:?}", self.cur_token.r#type);
                None
            }
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        // create Identifier
        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::ASSIGN) {
            return None;
        }

        // skip expression until semicolon
        while self.cur_token.r#type != TokenType::SEMICOLON {
            self.next_token();
        }

        Some(Statement::Let(LetStatement {
            token,
            name,
            value: Expression::Identifier(Identifier {
                token: Token::default(),
                value: "".to_string(),
            }),
        }))
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token.r#type == t {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.r#type
        );

        self.errors.push(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::Node, lexer::Lexer};

    fn check_parser_errors(p: &Parser) {
        if p.errors.is_empty() {
            return;
        }

        println!("parser has {} errors", p.errors.len());

        for msg in &p.errors {
            println!("parser error: {}", msg);
        }

        panic!("parser errors encountered");
    }

    #[test]
    fn test_let_statement() {
        let input = r#"
            let x = 5;
            let y = 10;
            let foobar = 383838;
            "#;

        let mut l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_programe();
        check_parser_errors(&p);

        let tests = vec!["x", "y", "foobar"];

        for (i, expected) in tests.iter().enumerate() {
            let stmt = &program.statements[i];

            test_let_statement(stmt, expected);
        }

        fn test_let_statement(stmt: &Statement, name: &str) {
            // check token literal
            assert_eq!(stmt.token_literal(), "let");

            // Rust enum match (替代 Go type assertion)
            match stmt {
                Statement::Let(let_stmt) => {
                    assert_eq!(let_stmt.name.value, name);
                    assert_eq!(let_stmt.name.token_literal(), name);
                }
            }
        }
    }
}
