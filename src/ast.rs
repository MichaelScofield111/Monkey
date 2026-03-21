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
/// let x = 10;
/// return x;
/// x + 10 + 1;
/// {....}
#[derive(Debug)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    BlockStatement(BlockStatement),
    // ... like Expression statement,
}

impl Node for Statement {
    fn token_literal(&self) -> &str {
        match self {
            Statement::Let(s) => s.token_literal(),
            Statement::Return(s) => s.token_literal(),
            Statement::Expression(s) => s.token_literal(),
            Statement::BlockStatement(s) => s.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Statement::Let(s) => s.string(),
            Statement::Return(s) => s.string(),
            Statement::Expression(s) => s.string(),
            Statement::BlockStatement(s) => s.string(),
        }
    }
}

// Expression
// let x = 5  x: Indentifier,  5: Expression
#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    Infix(InfixExpression),
    Prefix(PrefixExpression),
    Boolean(BooleanExpression),
    IfExpression(IfExpression),
    FunctionLiteral(FunctionLiteral),
    CallExpression(CallExpression),
}

impl Node for Expression {
    fn token_literal(&self) -> &str {
        match self {
            Expression::Identifier(i) => i.token_literal(),
            Expression::IntegerLiteral(i) => i.token_literal(),
            Expression::Infix(i) => i.token_literal(),
            Expression::Prefix(i) => i.token_literal(),
            Expression::Boolean(i) => i.token_literal(),
            Expression::IfExpression(i) => i.token_literal(),
            Expression::FunctionLiteral(i) => i.token_literal(),
            Expression::CallExpression(i) => i.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Expression::Identifier(i) => i.string(),
            Expression::IntegerLiteral(i) => i.string(),
            Expression::Infix(i) => i.string(),
            Expression::Prefix(i) => i.string(),
            Expression::Boolean(i) => i.string(),
            Expression::IfExpression(i) => i.string(),
            Expression::FunctionLiteral(i) => i.string(),
            Expression::CallExpression(i) => i.string(),
        }
    }
}

// ── Identifier
// let x = 5  x: Indentifier,  5: Expression
#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        self.statements
            .iter()
            .map(|s| s.string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Expression>,
    pub body: Box<BlockStatement>,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        format!(
            "fn({}) {}",
            self.parameters
                .iter()
                .map(|p| p.string())
                .collect::<Vec<_>>()
                .join(", "),
            self.body.string()
        )
    }
}

//  InfixExpression
// 例如: 5 + 3,  x * y,  a == b
#[derive(Debug)]
pub struct InfixExpression {
    pub token: Token,         // 运算符 token，比如 {Plus, "+"}
    pub lhs: Box<Expression>, // 左边
    pub op: String,           // 运算符字符串: "+", "-", "*", "==", etc.
    pub rhs: Box<Expression>, // 右边
}

impl Node for InfixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        format!("({} {} {})", self.lhs.string(), self.op, self.rhs.string())
    }
}

//  PrefixExpression
// 例如: -5,  !true
#[derive(Debug)]
pub struct PrefixExpression {
    pub token: Token,         // 运算符 token，比如 {Minus, "-"}
    pub op: String,           // 运算符字符串: "-", "!"
    pub rhs: Box<Expression>, // 右边的表达式
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        format!("({}{})", self.op, self.rhs.string())
    }
}

#[derive(Debug)]
pub struct BooleanExpression {
    pub token: Token,
    pub value: bool,
}

impl Node for BooleanExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(Debug)]
pub struct CallExpression {
    pub token: Token,
    pub arugument: Vec<Expression>,
    pub function: Box<Expression>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        format!(
            "{}({})",
            self.function.string(),
            self.arugument
                .iter()
                .map(|e| e.string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// if <condition> <expressionStatement> else <expressionStatement>
#[derive(Debug)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub if_block: Box<BlockStatement>,
    pub else_block: Option<Box<BlockStatement>>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    fn string(&self) -> String {
        let mut result = format!(
            "if ({}) \n {{{}}} \n",
            self.condition.string(),
            self.if_block.string()
        );
        if let Some(else_block) = &self.else_block {
            result.push_str(&format!(" else {{\n{}\n}}", else_block.string()));
        }
        result
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
