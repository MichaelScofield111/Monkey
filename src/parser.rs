use crate::{
    ast::{Identifier, LetStatement, Program, ReturnStatement, Statement},
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

    fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token.r#type != TokenType::Eof {
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
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            _ => {
                println!("unsupported type: {:?}", self.cur_token.r#type);
                None
            }
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        // Let Token
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return None;
        }

        // create Identifier
        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        // skip expression until semicolon
        while self.cur_token.r#type != TokenType::Semicolon {
            self.next_token();
        }

        Some(Statement::Let(LetStatement {
            token,
            name,
            value: None, // None 而不是假数据
        }))
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        // Return Token
        let token = self.cur_token.clone();

        self.next_token(); // 跳过 return，移到表达式第一个 token

        // TODO: 暂时跳过表达式，直到遇到分号
        while self.cur_token.r#type != TokenType::Semicolon
            && self.cur_token.r#type != TokenType::Eof
        {
            self.next_token();
        }

        Some(Statement::Return(ReturnStatement { token, value: None }))
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
        for msg in p.errors.iter() {
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

        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&p);

        assert_eq!(
            program.statements.len(),
            3,
            "expected 3 statements, got {}",
            program.statements.len()
        );

        let expected_names = ["x", "y", "foobar"];
        for (i, name) in expected_names.iter().enumerate() {
            check_let_statement(&program.statements[i], name);
        }
    }

    fn check_let_statement(stmt: &Statement, expected_name: &str) {
        assert_eq!(stmt.token_literal(), "let");
        match stmt {
            Statement::Let(let_stmt) => {
                assert_eq!(let_stmt.name.value, expected_name);
                assert_eq!(let_stmt.name.token_literal(), expected_name);
            }
            _ => panic!("expected LetStatement"),
        }
    }

    #[test]
    fn test_return_statement() {
        let input = r#"
                return 5;
                return 10;
                return 993322;
            "#;

        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&p);

        assert_eq!(
            program.statements.len(),
            3,
            "expected 3 statements, got {}",
            program.statements.len()
        );

        for stmt in &program.statements {
            assert_eq!(stmt.token_literal(), "return");
            match stmt {
                Statement::Return(_) => {}
                _ => panic!("expected ReturnStatement"),
            }
        }
    }
}
