use crate::token::Token;

// Node is a ability
pub trait Node {
    fn token_literal(&self) -> &str;
}

// Programe == File and Statement like sub node
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Node for Program {
    fn token_literal(&self) -> &str {
        if let Some(stmt) = self.statements.first() {
            stmt.token_literal()
        } else {
            ""
        }
    }
}

pub enum Statement {
    Let(LetStatement),
}

impl Node for Statement {
    fn token_literal(&self) -> &str {
        match self {
            Statement::Let(s) => s.token_literal(),
        }
    }
}

// let x = 5  x: Indentifier,  5: Expression
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

pub enum Expression {
    Identifier(Identifier),
}

impl Node for Expression {
    fn token_literal(&self) -> &str {
        match self {
            Expression::Identifier(i) => i.token_literal(),
        }
    }
}

// let x = 5
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

impl Node for LetStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}
