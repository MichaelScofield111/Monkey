use crate::{
    ast::{Expression, IfExpression, Program, Statement},
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
        // parser if() {..} else {..}
        Expression::IfExpression(i) => eval_ifexpression(i),
        Expression::Prefix(prefix) => {
            let rhs = eval_expression(&prefix.rhs)?;
            eval_prefix_expression(&prefix.op, rhs)
        }
        Expression::Infix(infix) => {
            let lhs = eval_expression(&infix.lhs)?;
            let rhs = eval_expression(&infix.rhs)?;
            eval_infix_expression(&infix.op, lhs, rhs)
        }
        _ => Err(format!("unsupported expression type: {:?}", expr)),
    }
}

fn eval_prefix_expression(op: &str, rhs: Object) -> Result<Object, String> {
    match op {
        "!" => match rhs {
            Object::Boolean(b) => Ok(Object::Boolean(Boolean { value: !b.value })),
            Object::Null(_) => Ok(Object::Boolean(Boolean { value: true })),
            Object::Integer(i) => Ok(Object::Boolean(Boolean {
                value: i.value == 0,
            })), // !0 => true, !非0 => false
        },
        "-" => match rhs {
            Object::Integer(i) => Ok(Object::Integer(Integer { value: -i.value })),
            other => Err(format!("expected integer after '-', got: {:?}", other)),
        },
        _ => Err(format!("unsupported prefix operator: {}", op)),
    }
}

// a + b
fn eval_infix_expression(op: &str, lhs: Object, rhs: Object) -> Result<Object, String> {
    match (&lhs, &rhs) {
        (Object::Integer(l), Object::Integer(r)) => eval_infix_integer(l, r, op),
        (Object::Boolean(l), Object::Boolean(r)) => eval_infix_boolean(l, r, op),
        _ => Err(format!(
            "expected integer operands for infix operator: {}",
            op
        )),
    }
}

// a + b
fn eval_infix_integer(left: &Integer, right: &Integer, op: &str) -> Result<Object, String> {
    match op {
        "+" => Ok(Object::Integer(Integer {
            value: left.value + right.value,
        })),
        "-" => Ok(Object::Integer(Integer {
            value: left.value - right.value,
        })),
        "*" => Ok(Object::Integer(Integer {
            value: left.value * right.value,
        })),
        "/" => Ok(Object::Integer(Integer {
            value: left.value / right.value,
        })),
        "==" => Ok(Object::Boolean(Boolean {
            value: left.value == right.value,
        })),
        "!=" => Ok(Object::Boolean(Boolean {
            value: left.value != right.value,
        })),
        "<" => Ok(Object::Boolean(Boolean {
            value: left.value < right.value,
        })),
        ">" => Ok(Object::Boolean(Boolean {
            value: left.value > right.value,
        })),
        _ => Err(format!("unsupported infix operator: {}", op)),
    }
}

// if (condition) {..} else {..}
fn eval_ifexpression(ep: &IfExpression) -> Result<Object, String> {
    // condition
    // - integer
    // - boolean
    // - null
    let condition = eval_expression(&ep.condition)?;
    // if(1 + 1) {
    //  if (...) {
    //   }
    // }
    if is_true(&condition) {
        return eval_statements(&ep.if_block.statements);
    } else {
        if let Some(x) = ep.else_block.as_ref() {
            return eval_statements(&x.statements);
        } else {
            Ok(Object::Null(Null {})) // 没有 else 块就返回 Null
        }
    }
}

fn is_true(obj: &Object) -> bool {
    match obj {
        Object::Boolean(x) => x.value,
        Object::Integer(x) => return x.value != 0,
        Object::Null(_x) => {
            return false;
        }
    }
}

fn eval_infix_boolean(l: &Boolean, r: &Boolean, op: &str) -> Result<Object, String> {
    match op {
        "==" => Ok(Object::Boolean(Boolean {
            value: l.value == r.value,
        })),
        "!=" => Ok(Object::Boolean(Boolean {
            value: l.value != r.value,
        })),
        _ => Err(format!("unsupported infix operator: {}", op)),
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

    fn assert_boolean_object(obj: &Object, expected: bool) {
        match obj {
            Object::Boolean(b) => {
                assert_eq!(b.value, expected, "boolean value mismatch");
            }
            other => panic!("expected Boolean object, got: {:?}", other),
        }
    }

    #[test]
    fn test_eval_bang() {
        struct TestCase {
            input: &'static str,
            expected: bool,
            has_error: bool,
        }
        let tests = vec![
            TestCase {
                input: "!true",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "!false",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "!5",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "!0",
                expected: true,
                has_error: false,
            },
        ];

        for tc in &tests {
            let result = string_to_ast(tc.input);
            assert_eq!(
                tc.has_error,
                result.is_err(),
                "expected error: {}",
                tc.input
            );
            if let Ok(obj) = result {
                assert_boolean_object(&obj, tc.expected);
            }
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
                input: "-5",
                expected: -5,
                has_error: false,
            },
            TestCase {
                input: "-10",
                expected: -10,
                has_error: false,
            },
            TestCase {
                input: "1+2+3+4-10",
                expected: 0,
                has_error: false,
            },
            TestCase {
                input: "-1+2*-2",
                expected: -5,
                has_error: false,
            },
            TestCase {
                input: "10/2*3",
                expected: 15,
                has_error: false,
            },
            TestCase {
                input: "(1+3)*-4",
                expected: -16,
                has_error: false,
            },
            TestCase {
                input: "(4+3)*(4)+-29",
                expected: -1,
                has_error: false,
            },
            TestCase {
                input: "111111111111111111111111111111111111",
                expected: 0,
                has_error: true,
            }, // 溢出应报错
            TestCase {
                input: "-true",
                expected: 0,
                has_error: true,
            }, // should report an error
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

    #[test]
    fn test_eval_boolean() {
        struct TestCase {
            input: &'static str,
            expected: bool,
            has_error: bool,
        }

        let tests = vec![
            TestCase {
                input: "true",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "false",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "true == true",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "false == false",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "true != false",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "false != true",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "true != true",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "false != false",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "true == false",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "false == true",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "1 < 2",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "1 > 2",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "2 == 2",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "2 != 2",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "2 == (1+1)",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "3 == 2 * (1+1)",
                expected: false,
                has_error: false,
            },
            TestCase {
                input: "3 != 2 * (1+1)",
                expected: true,
                has_error: false,
            },
            TestCase {
                input: "TRUE",
                expected: false,
                has_error: true,
            }, // should report an error
            TestCase {
                input: "false < true",
                expected: false,
                has_error: true,
            },
            TestCase {
                input: "false > true",
                expected: false,
                has_error: true,
            },
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
                assert_boolean_object(&obj, tc.expected);
            }
        }
    }

    #[test]
    fn test_eval_ifelse() {
        struct TestCase {
            input: &'static str,
            expected: Option<i64>, // None 表示应该返回 Null
            has_error: bool,
        }

        let tests = vec![
            // 条件为 true，执行 if 块
            TestCase {
                input: "if (true) { 10 }",
                expected: Some(10),
                has_error: false,
            },
            // 条件为 false，没有 else，返回 Null
            TestCase {
                input: "if (false) { 10 }",
                expected: None,
                has_error: false,
            },
            // 条件为 false，有 else
            TestCase {
                input: "if (false) { 10 } else { 20 }",
                expected: Some(20),
                has_error: false,
            },
            // 条件为 true，有 else，走 if 分支
            TestCase {
                input: "if (true) { 10 } else { 20 }",
                expected: Some(10),
                has_error: false,
            },
            // 条件是表达式
            TestCase {
                input: "if (1 < 2) { 10 }",
                expected: Some(10),
                has_error: false,
            },
            TestCase {
                input: "if (1 > 2) { 10 }",
                expected: None,
                has_error: false,
            },
            TestCase {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: Some(10),
                has_error: false,
            },
            TestCase {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: Some(20),
                has_error: false,
            },
            // 整数作为条件：非 0 为 true
            TestCase {
                input: "if (1) { 10 }",
                expected: Some(10),
                has_error: false,
            },
            TestCase {
                input: "if (0) { 10 } else { 20 }",
                expected: Some(20),
                has_error: false,
            },
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
                match tc.expected {
                    Some(expected) => assert_integer_object(&obj, expected),
                    None => assert!(
                        matches!(obj, Object::Null(_)),
                        "input='{}', expected Null, got: {:?}",
                        tc.input,
                        obj
                    ),
                }
            }
        }
    }
}
