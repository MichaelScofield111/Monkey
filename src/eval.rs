use crate::{
    ast::{Expression, Program, Statement},
    object::{Boolean, Integer, Null, Object},
};

pub fn eval(ast: &Program) -> Result<Object, String> {
    eval_statements(&ast.statements)
}

fn eval_statements(stmts: &Vec<Statement>) -> Result<Object, String> {
    let mut result = Object::Null(Null {});
    for stmt in stmts {
        result = eval_statement(stmt)?;
    }

    Ok(result)
}

fn eval_statement(stmt: &Statement) -> Result<Object, String> {
    match stmt {
        Statement::Expression(es) => match &es.expression {
            Some(expr) => eval_expression(expr),
            None => Ok(Object::Null(Null {})),
        },
        Statement::Let(_) => Ok(Object::Null(Null {})),
        Statement::Return(_) => Ok(Object::Null(Null {})),
        Statement::BlockStatement(bs) => eval_statements(&bs.statements),
    }
}

fn eval_expression(expr: &Expression) -> Result<Object, String> {
    match expr {
        // integer
        Expression::IntegerLiteral(i) => Ok(Object::Integer(Integer { value: i.value })),
        Expression::Boolean(i) => Ok(Object::Boolean(Boolean { value: i.value })),
        _ => Err(format!("unsupported expression type: {:?}", expr)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, object::Object, parser::Parser};

    fn string_to_ast(input: &str) -> Result<Object, String> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        // 检查 parser 错误
        let errors = parser.errors;
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }

        eval(&program)
    }

    fn assert_integer_object(obj: &Object, expected: i64) {
        match obj {
            Object::Integer(i) => {
                assert_eq!(i.value, expected, "integer value mismatch");
            }
            other => panic!("expected Integer object, got: {:?}", other),
        }
    }

    #[test]
    fn test_eval_integer() {
        struct TestCase {
            input: &'static str,
            expected: i64,
            has_error: bool,
        }

        let tests = vec![
            TestCase {
                input: "5",
                expected: 5,
                has_error: false,
            },
            TestCase {
                input: "123",
                expected: 123,
                has_error: false,
            },
            TestCase {
                input: "111111111111111111111111111111111111",
                expected: 0,
                has_error: true,
            }, // 溢出应报错
        ];

        for tc in &tests {
            let result = string_to_ast(tc.input);
            assert_eq!(
                tc.has_error,
                result.is_err(),
                "input='{}', err={:?}",
                tc.input,
                result
            );

            if let Ok(obj) = result {
                assert_integer_object(&obj, tc.expected);
            }
        }
    }
}
