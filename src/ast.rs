use crate::token::Token;

// Node is a ability
pub trait Node {
    fn token_literal(&self) -> &str;
    fn string(&self) -> String;
}

// Programe == File and Statement like sub node
// root
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Node for Program {
    fn token_literal(&self) -> &str {
        self.statements
            .first()
            .map(|s| s.token_literal())
            .unwrap_or("")
    }
    fn string(&self) -> String {
        self.statements.iter().map(|s| s.string()).collect()
    }
}

// Statement
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    // ... like Expression statement,
}

impl Node for Statement {
    fn token_literal(&self) -> &str {
        match self {
            Statement::Let(s) => s.token_literal(),
            Statement::Return(s) => s.token_literal(),
            Statement::Expression(s) => s.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Statement::Let(s) => s.string(),
            Statement::Return(s) => s.string(),
            Statement::Expression(s) => s.string(),
        }
    }
}

// Expression
// let x = 5  x: Indentifier,  5: Expression
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    // any Expression will be add
}

impl Node for Expression {
    fn token_literal(&self) -> &str {
        match self {
            Expression::Identifier(i) => i.token_literal(),
            Expression::IntegerLiteral(i) => i.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Identifier(i) => i.string(),
            Expression::IntegerLiteral(i) => i.string(),
        }
    }
}

// ── Identifier
// let x = 5  x: Indentifier,  5: Expression
pub struct Identifier {
    pub token: Token, // IDENT token
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.clone()
    }
}

pub struct IntegerLiteral {
    pub token: Token, // {INT, 5}
    pub value: i64,   // 5 -> 5
}
impl Node for IntegerLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

// ── let <name> = <value>;
pub struct LetStatement {
    pub token: Token, // LET token
    pub name: Identifier,
    /// `None` = 表达式部分暂时跳过（TODO 阶段），不再用假数据填充
    pub value: Option<Expression>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "let {} = {};",
            self.name.string(),
            self.value.as_ref().map(|v| v.string()).unwrap_or_default()
        )
    }
}

// ── return <value>;
pub struct ReturnStatement {
    pub token: Token, // RETURN token
    /// `None` = 表达式部分暂时跳过
    pub value: Option<Expression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "return {};",
            self.value.as_ref().map(|v| v.string()).unwrap_or_default()
        )
    }
}

// ExpressionStatement
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Option<Expression>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        self.expression
            .as_ref()
            .map(|e| e.string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_string() {
        // 手动构造 `let myVar = anotherVar;`，验证 string() 输出正确
        let program = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token {
                    r#type: TokenType::Let,
                    literal: "let".to_string(),
                },
                name: Identifier {
                    token: Token {
                        r#type: TokenType::Ident,
                        literal: "myVar".to_string(),
                    },
                    value: "myVar".to_string(),
                },
                value: Some(Expression::Identifier(Identifier {
                    token: Token {
                        r#type: TokenType::Ident,
                        literal: "anotherVar".to_string(),
                    },
                    value: "anotherVar".to_string(),
                })),
            })],
        };

        assert_eq!(program.string(), "let myVar = anotherVar;");
    }
}
