use crate::{
    ast::{
        BlockStatement, BooleanExpression, CallExpression, Expression, ExpressionStatement,
        FunctionLiteral, Identifier, IfExpression, InfixExpression, IntegerLiteral, LetStatement,
        PrefixExpression, Program, ReturnStatement, Statement,
    },
    lexer::Lexer,
    token::{Token, TokenType},
};

// Pratt Parser
pub enum Precedence {
    Lowest = 1,
    Equals, // ==
    // flase == (2 < 3)
    LessGreater, // > or <
    // 1 + (2 * 3)
    Sum,
    Product,
    // (-X) * Y: -X is first Precedence
    Prefix, // -X or !X
    Call,   // fn()
}

impl Precedence {
    fn as_num(&self) -> u8 {
        match self {
            Precedence::Lowest => 1,
            Precedence::Equals => 2,
            Precedence::LessGreater => 3,
            Precedence::Sum => 4,
            Precedence::Product => 5,
            Precedence::Prefix => 6,
            Precedence::Call => 7,
        }
    }

    fn from_num(n: u8) -> Self {
        match n {
            2 => Precedence::Equals,
            3 => Precedence::LessGreater,
            4 => Precedence::Sum,
            5 => Precedence::Product,
            6 => Precedence::Prefix,
            7 => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a>, // parser has lexer ownership

    pub cur_token: Token,
    pub peek_token: Token,
    pub errors: Vec<String>,
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

    fn token_precedence(tt: &TokenType) -> u8 {
        match tt {
            TokenType::Eq | TokenType::Noteq => Precedence::Equals.as_num(),
            TokenType::Lt | TokenType::Gt => Precedence::LessGreater.as_num(),
            TokenType::Plus | TokenType::Minus => Precedence::Sum.as_num(),
            TokenType::Asterisk | TokenType::Slash => Precedence::Product.as_num(),
            TokenType::Lparen => Precedence::Call.as_num(),
            _ => Precedence::Lowest.as_num(),
        }
    }

    fn peek_precedence(&self) -> u8 {
        Self::token_precedence(&self.peek_token.r#type)
    }

    fn cur_precedence(&self) -> u8 {
        Self::token_precedence(&self.cur_token.r#type)
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
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

    /// Let x = 10 + 10 + 5;
    /// parse_let_statement => LetStatement { token: Let Token, name: x, value: Some(Expression) }
    ///
    /// a * b + c + d * c
    /// parse_expression_statement =>
    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.r#type {
            TokenType::Let => self.parse_let_statement().map(Statement::Let),
            TokenType::Return => self.parse_return_statement().map(Statement::Return),
            _ => self.parse_expression_statement().map(Statement::Expression),
        }
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
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

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);
        if !self.expect_peek(TokenType::Semicolon) {
            return None;
        }

        Some(LetStatement { token, name, value })
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        // Return Token
        let token = self.cur_token.clone();

        self.next_token(); // 跳过 return，移到表达式第一个 token

        let value = self.parse_expression(Precedence::Lowest);
        if !self.expect_peek(TokenType::Semicolon) {
            return None;
        }

        Some(ReturnStatement { token, value })
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

    /// a * b + c + d * c
    /// parse_expression_statement =>
    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement> {
        let stmt = Some(ExpressionStatement {
            token: self.cur_token.clone(),
            expression: self.parse_expression(Precedence::Lowest),
        });

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        stmt
    }

    ///  表达式解析（Pratt 解析器核心)
    /// a * b + c + d * c
    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        // when token is left we need call parse_prefix()
        // left = a
        let mut left = self.parse_prefix()?;

        // if current precedence is lower than peek precedence, parse infix expression
        while !self.peek_token_is(&TokenType::Semicolon)
            && precedence.as_num() < self.peek_precedence()
        {
            match self.peek_token.r#type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Eq
                | TokenType::Noteq
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::Lparen => {
                    self.next_token(); // cur becomes the operator
                    // left need to cal with next token
                    left = self.parse_infix_expression(left)?;
                }
                _ => break,
            }
        }

        Some(left)
    }

    /// Infix expression: `<lhs> OP <rhs>`
    ///
    /// cur = operator (+, -, *, /, ==, !=, <, >)
    /// lhs has already been parsed and is passed in.
    fn parse_infix_expression(&mut self, lhs: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let op = self.cur_token.literal.clone();

        // -add(10 + 2)
        // lhs = add
        if self.current_token_is(&TokenType::Lparen) {
            return self.parse_function_call(lhs);
        }

        // Remember *this* operator's precedence as the new floor for rhs.
        // Using the same precedence (not +1) gives left-associativity:
        //   a + b + c  →  ((a + b) + c)
        // because the second + is NOT strictly greater than Sum, so it
        // does NOT get consumed by the inner parseExpression call.
        let cur_prec = Precedence::from_num(self.cur_precedence());

        self.next_token(); // move past operator; cur = first token of rhs

        let rhs = self.parse_expression(cur_prec)?;

        Some(Expression::Infix(InfixExpression {
            token,
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    }

    /// 根据当前 token 调用对应的「前缀解析函数」
    fn parse_prefix(&mut self) -> Option<Expression> {
        match self.cur_token.r#type {
            TokenType::Ident => self.parse_identifier(),
            TokenType::Int => self.parse_integer_literal(),
            TokenType::Bang => self.parse_prefix_expression(),
            TokenType::Minus => self.parse_prefix_expression(),
            TokenType::False => self.parse_boolean(),
            TokenType::True => self.parse_boolean(),
            TokenType::Lparen => self.parse_grouped_expression(),
            _ => {
                let msg = format!(
                    "no prefix parse function for {:?} found",
                    self.cur_token.r#type
                );
                self.errors.push(msg);
                None
            }
        }
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    /// 前缀函数：整数字面量
    fn parse_integer_literal(&mut self) -> Option<Expression> {
        match self.cur_token.literal.parse::<i64>() {
            Ok(value) => Some(Expression::IntegerLiteral(IntegerLiteral {
                token: self.cur_token.clone(),
                value,
            })),
            Err(_) => {
                let msg = format!("could not parse {:?} as integer", self.cur_token.literal);
                self.errors.push(msg);
                None
            }
        }
    }

    fn parse_boolean(&mut self) -> Option<Expression> {
        let bool_value = self.cur_token.literal.parse::<bool>().unwrap_or(true);
        Some(Expression::Boolean(BooleanExpression {
            token: self.cur_token.clone(),
            value: bool_value,
        }))
    }

    /// Prefix expression: `!<rhs>` or `-<rhs>`
    ///
    /// cur = operator (!  or -)
    /// After parsing: cur = last token of rhs
    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let op = self.cur_token.literal.clone();
        self.next_token(); // move past operator; cur = first token of rhs

        let rhs = self.parse_expression(Precedence::Prefix)?;

        Some(Expression::Prefix(PrefixExpression {
            token,
            op,
            rhs: Box::new(rhs),
        }))
    }

    /// let foo = if (x>y) {x} return {y};
    /// if (<condition>) {<block>} else {<block>}
    fn parse_ifelse_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::Lparen) {
            return None;
        }
        self.next_token();

        // parse condition
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        if !self.expect_peek(TokenType::Lbrace) {
            return None;
        }

        // if block
        let if_block = self.parse_block_statement()?;

        // else
        let else_block = self
            .peek_token_is(&TokenType::Else)
            .then(|| {
                self.next_token();
                self.expect_peek(TokenType::Lbrace);
                Some(Box::new(self.parse_block_statement()?))
            })
            .flatten();

        Some(Expression::IfExpression(IfExpression {
            token,
            condition: Box::new(condition),
            if_block: Box::new(if_block),
            else_block,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let mut block = BlockStatement {
            token: self.cur_token.clone(), // "{"
            statements: Vec::new(),
        };

        self.next_token();

        while !self.current_token_is(&TokenType::Rbrace) && !self.current_token_is(&TokenType::Eof)
        {
            let stmt = self.parse_statement();
            if let Some(stmt) = stmt {
                block.statements.push(stmt);
            } else {
                return None;
            }
        }

        Some(block)
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Expression>> {
        if self.peek_token_is(&TokenType::Rparen) {
            // no parameters
            self.next_token();
            return None;
        }

        // to get parameters
        self.next_token();
        let mut parameters = Vec::new();
        if let Some(v) = self.parse_identifier() {
            parameters.push(v);
        }

        while self.peek_token_is(&TokenType::Comma) {
            // eat comma
            self.next_token();
            // skip comma
            self.next_token();
            if let Some(v) = self.parse_identifier() {
                parameters.push(v);
            }
        }

        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        Some(parameters)
    }

    // fn <Params list> <Body>
    // <Params List> = (Indentifier1, Indentifier2...)
    // let f = fn(a, b) {return a + b}
    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if self.expect_peek(TokenType::Lparen) {
            return None;
        }

        let parameters = self.parse_function_parameters();
        if !self.expect_peek(TokenType::Lbrace) {
            return None;
        }

        let body = self.parse_block_statement();

        //TODO remove
        if !self.current_token_is(&TokenType::Rbrace) {
            panic!("the {{ is no close");
        }

        Some(Expression::FunctionLiteral(FunctionLiteral {
            token,
            parameters: parameters.unwrap(),
            body: Box::new(body.unwrap()),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        // read "("
        self.next_token();

        let grouped = self.parse_expression(Precedence::Lowest);
        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        grouped
    }

    fn peek_token_is(&self, toke_type: &TokenType) -> bool {
        &self.peek_token.r#type == toke_type
    }

    fn current_token_is(&self, toke_type: &TokenType) -> bool {
        &self.cur_token.r#type == toke_type
    }

    // parseFunctionCall is a special case for parseInfixExpression
    fn parse_function_call(&mut self, function: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let args = self.parse_call_arguments().unwrap_or_default();
        Some(Expression::CallExpression(CallExpression {
            token,
            arugument: args,
            function: Box::new(function),
        }))
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
        // add(1, add(1+2), 3)
        if self.peek_token_is(&TokenType::Rparen) {
            // no params
            self.next_token();
            return None;
        }

        // "("
        self.next_token();
        let mut args = vec![];

        if let Some(arg) = self.parse_expression(Precedence::Lowest) {
            args.push(arg);
        }

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            if let Some(arg) = self.parse_expression(Precedence::Lowest) {
                args.push(arg);
            }
        }

        if !self.expect_peek(TokenType::Rparen) {
            return None;
        }

        Some(args)
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

    #[test]
    fn test_prefix_expressions() {
        let cases = vec![
            ("!a;", "!", "a"),
            ("-5;", "-", "5"),
            ("!true;", "!", "true"),
            ("!false;", "!", "false"),
        ];

        for (input, expected_op, expected_rhs) in cases {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&p);

            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::Expression(es) => match &es.expression {
                    Some(Expression::Prefix(pre)) => {
                        assert_eq!(pre.op, expected_op);
                        assert_eq!(pre.rhs.string(), expected_rhs);
                    }
                    _ => panic!("expected PrefixExpression for input: {}", input),
                },
                _ => panic!("expected ExpressionStatement"),
            }
        }
    }

    #[test]
    fn test_infix_expressions() {
        let cases: Vec<(&str, &str, &str, &str)> = vec![
            ("a + b;", "a", "+", "b"),
            ("a - b;", "a", "-", "b"),
            ("a * b;", "a", "*", "b"),
            ("a / b;", "a", "/", "b"),
            ("a < b;", "a", "<", "b"),
            ("a > b;", "a", ">", "b"),
            ("a == b;", "a", "==", "b"),
            ("a != b;", "a", "!=", "b"),
        ];

        for (input, expected_lhs, expected_op, expected_rhs) in cases {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&p);

            assert_eq!(program.statements.len(), 1);
            match &program.statements[0] {
                Statement::Expression(es) => match &es.expression {
                    Some(Expression::Infix(inf)) => {
                        assert_eq!(inf.lhs.string(), expected_lhs);
                        assert_eq!(inf.op, expected_op);
                        assert_eq!(inf.rhs.string(), expected_rhs);
                    }
                    _ => panic!("expected InfixExpression for input: {}", input),
                },
                _ => panic!("expected ExpressionStatement"),
            }
        }
    }

    // ── operator precedence (verified via string() which adds parens)

    #[test]
    fn test_operator_precedence() {
        let cases = vec![
            // higher precedence binds tighter
            ("a + b * c;", "(a + (b * c))"),
            ("a * b + c;", "((a * b) + c)"),
            // classic case from the doc comment
            ("a * b + c * d;", "((a * b) + (c * d))"),
            // left-associativity: same precedence folds left
            ("a + b + c;", "((a + b) + c)"),
            ("a - b - c;", "((a - b) - c)"),
            // prefix binds tightest
            ("!-a;", "(!(-a))"),
            ("-a + b;", "((-a) + b)"),
            // comparison
            ("a + b == c + d;", "((a + b) == (c + d))"),
        ];

        for (input, expected) in cases {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&p);
            assert_eq!(program.string(), expected, "input: {}", input);
        }
    }

    #[test]
    fn test_boolean_expression() {
        let cases = vec![("true;", true), ("false;", false)];

        for (input, expected_value) in cases {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&p);

            assert_eq!(
                program.statements.len(),
                1,
                "expected 1 statement, got {} for input: {}",
                program.statements.len(),
                input
            );

            match &program.statements[0] {
                Statement::Expression(es) => match &es.expression {
                    Some(Expression::Boolean(b)) => {
                        assert_eq!(
                            b.value, expected_value,
                            "expected bool value {} for input: {}",
                            expected_value, input
                        );
                    }
                    other => panic!(
                        "expected BooleanExpression, got {:?} for input: {}",
                        other, input
                    ),
                },
                other => panic!("expected ExpressionStatement, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_call_expression() {
        let cases = vec![
            (
                "add(1, 2+3, 4 + 5*6, 7*8+10)",
                "add(1,(2 + 3),(4 + (5 * 6)),((7 * 8) + 10))",
            ),
            ("non()", "non()"),
            ("negate(1)", "negate(1)"),
        ];

        for (input, expected) in cases {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&p);

            assert_eq!(
                program.statements.len(),
                1,
                "expected 1 statement, got {} for input: {}",
                program.statements.len(),
                input
            );

            match &program.statements[0] {
                Statement::Expression(es) => match &es.expression {
                    Some(Expression::CallExpression(call)) => {
                        assert_eq!(call.string(), expected, "input: {}", input);
                    }
                    other => panic!(
                        "expected CallExpression, got {:?} for input: {}",
                        other, input
                    ),
                },
                other => panic!("expected ExpressionStatement, got {:?}", other),
            }
        }
    }
}
