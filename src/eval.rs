use crate::{
    ast::{Expression, Identifier, IfExpression, Program, Statement},
    builtin::get_builtin,
    environment::Environment,
    object::{Array, Boolean, Function, HashKey, HashObject, HashPair, Integer, MonString, Null, Object, ReturnValue},
};
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

pub fn eval(ast: &Program, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    eval_statements(&ast.statements, env)
}

fn eval_statements(
    stmts: &Vec<Statement>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut result = Object::Null(Null {});
    for stmt in stmts {
        result = eval_statement(stmt, env.clone())?;

        // top level
        if let Object::ReturnValue(rv) = result {
            return Ok(*rv.value);
        }
    }

    Ok(result)
}

fn eval_statement(stmt: &Statement, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match stmt {
        Statement::Expression(es) => match &es.expression {
            Some(expr) => eval_expression(expr, env.clone()),
            None => Ok(Object::Null(Null {})),
        },
        Statement::Let(ls) => {
            let val = match &ls.value {
                Some(expr) => eval_expression(expr, env.clone())?,
                None => return Err(format!("let statement missing value {}", ls.name.value)),
            };
            env.borrow_mut().set(&ls.name.value, val);
            Ok(Object::Null(Null {}))
        }
        Statement::Return(rs) => match &rs.value {
            Some(expr) => {
                let val = eval_expression(expr, env.clone())?;
                Ok(Object::ReturnValue(ReturnValue {
                    value: Box::new(val), // Box::new 包装
                }))
            }
            None => Ok(Object::ReturnValue(ReturnValue {
                value: Box::new(Object::Null(Null {})),
            })),
        },
        Statement::BlockStatement(bs) => eval_block_statement(&bs.statements, env.clone()),
    }
}

fn eval_expression(expr: &Expression, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match expr {
        // integer
        Expression::IntegerLiteral(i) => Ok(Object::Integer(Integer { value: i.value })),
        Expression::Boolean(i) => Ok(Object::Boolean(Boolean { value: i.value })),
        // parse to bind identifier
        Expression::Identifier(i) => eval_identifier(i, env.clone()),
        // parser if() {..} else {..}
        Expression::IfExpression(i) => eval_ifexpression(i, env.clone()),
        Expression::FunctionLiteral(fl) => {
            let mut params = Vec::with_capacity(fl.parameters.len());
            for param in &fl.parameters {
                match param {
                    Expression::Identifier(ident) => params.push(ident.clone()),
                    other => {
                        return Err(format!(
                            "function parameter must be identifier, got: {:?}",
                            other
                        ));
                    }
                }
            }

            Ok(Object::Function(Function {
                params,
                body: fl.body.as_ref().clone(),
                env: env.clone(),
            }))
        }
        Expression::Prefix(prefix) => {
            let rhs = eval_expression(&prefix.rhs, env.clone())?;
            eval_prefix_expression(&prefix.op, rhs)
        }
        Expression::Infix(infix) => {
            let lhs = eval_expression(&infix.lhs, env.clone())?;
            let rhs = eval_expression(&infix.rhs, env.clone())?;
            eval_infix_expression(&infix.op, lhs, rhs)
        }
        Expression::CallExpression(ce) => {
            let f = eval_expression(&ce.function, env.clone())?;
            let args = eval_expressions(&ce.arugument, env.clone())?;
            call_function(f, args)
        }
        Expression::StringLiteral(se) => Ok(Object::MonString(MonString {
            value: se.value.clone(),
        })),
        Expression::ArrayLiteral(ae) => {
            let args = eval_expressions(&ae.elements, env.clone())?;
            Ok(Object::Array(Array::new(args)))
        }
        Expression::HashExpression(he) => eval_hash_literal(he, env.clone()),
        Expression::IndexExpression(ie) => {
            let left = eval_expression(&ie.left, env.clone())?;
            let index = eval_expression(&ie.index, env.clone())?;
            eval_index_expression(left, index)
        }
        _ => Err(format!("no support eval expression")),
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
            _ => Err(format!("unsupported prefix operator: {}", op)),
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
        (Object::MonString(_), Object::MonString(_)) => eval_infix_string(lhs, op, rhs),
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

fn eval_block_statement(
    stmts: &Vec<Statement>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut result = Object::Null(Null {});
    for stmt in stmts {
        result = eval_statement(stmt, env.clone())?;
        // block 内不要解包，原样向外冒泡
        if let Object::ReturnValue(_) = result {
            return Ok(result);
        }
    }
    Ok(result)
}

// if (condition) {..} else {..}
fn eval_ifexpression(ep: &IfExpression, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    // condition
    // - integer
    // - boolean
    // - null
    let condition = eval_expression(&ep.condition, env.clone())?;
    // if(1 + 1) {
    //  if (...) {
    //   }
    // }
    if is_true(&condition) {
        eval_block_statement(&ep.if_block.statements, env.clone())
    } else if let Some(x) = ep.else_block.as_ref() {
        eval_block_statement(&x.statements, env.clone())
    } else {
        Ok(Object::Null(Null {})) // 没有 else 块就返回 Null
    }
}

fn is_true(obj: &Object) -> bool {
    match obj {
        Object::Boolean(x) => x.value,
        Object::Integer(x) => x.value != 0,
        Object::Null(_x) => false,
        _ => false,
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

fn eval_identifier(ident: &Identifier, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    if let Some(obj) = env.borrow().get(&ident.value) {
        return Ok(obj);
    }

    if let Some(builtin) = get_builtin(&ident.value) {
        return Ok(builtin);
    }

    Err(format!("identifier not found: {}", ident.value))
}

fn eval_expressions(
    args: &Vec<Expression>,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Object>, String> {
    let mut evaluated = Vec::new();
    for arg in args {
        let obj = eval_expression(arg, env.clone())?;
        evaluated.push(obj);
    }
    Ok(evaluated)
}

fn eval_infix_string(l: Object, op: &str, r: Object) -> Result<Object, String> {
    match (l, op, r) {
        (Object::MonString(l), "==", Object::MonString(r)) => Ok(Object::Boolean(Boolean {
            value: l.value == r.value,
        })),
        (Object::MonString(l), "!=", Object::MonString(r)) => Ok(Object::Boolean(Boolean {
            value: l.value != r.value,
        })),
        (Object::MonString(l), "+", Object::MonString(r)) => Ok(Object::MonString(MonString {
            value: l.value + &r.value,
        })),
        _ => Err("unsupported infix operator for strings".to_string()),
    }
}

/*
    fn(x, y) {

    }
    (1, 2) -> args
*/
fn call_function(f: Object, args: Vec<Object>) -> Result<Object, String> {
    match f {
        Object::Function(fun) => {
            if args.len() != fun.params.len() {
                return Err(format!(
                    "wrong number of arguments: expected {}, got {}",
                    fun.params.len(),
                    args.len()
                ));
            }

            let mut env = Environment::new_enclosed(fun.env.clone());
            for (param, arg) in fun.params.iter().zip(args.into_iter()) {
                env.set(&param.value, arg);
            }
            let evaluated = eval_block_statement(&fun.body.statements, Rc::new(RefCell::new(env)))?;
            if let Object::ReturnValue(rv) = evaluated {
                Ok(*rv.value)
            } else {
                Ok(evaluated)
            }
        }
        Object::Builtin(b) => b.call(args),
        _ => Err(format!("expected Function object, got: {:?}", f)),
    }
}

fn eval_index_expression(left: Object, index: Object) -> Result<Object, String> {
    match (left, index) {
        (Object::Array(arr), Object::Integer(i)) => eval_array_index_expression(arr, i.value),
        (Object::Array(_), other) => Err(format!(
            "the index should be an integer, but got {:?}",
            other
        )),
        (Object::Hash(hash), index) => eval_hash_index_expression(hash, index),
        (other, _) => Err(format!("{:?} is not indexable", other)),
    }
}

fn eval_array_index_expression(arr: Array, idx: i64) -> Result<Object, String> {
    if idx < 0 || idx as usize >= arr.len() {
        return Err(format!(
            "index out of bounds, len:{}, visit:{}",
            arr.len(),
            idx
        ));
    }

    arr.get(idx as usize)
        .ok_or_else(|| format!("index out of bounds, len:{}, visit:{}", arr.len(), idx))
}

fn eval_hash_literal(
    he: &crate::ast::HashLiteral,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut pairs: HashMap<HashKey, HashPair> = HashMap::new();
    for (key_expr, value_expr) in &he.pairs {
        let key = eval_expression(key_expr, env.clone())?;
        let hash_key = object_to_hash_key(&key)?;
        let value = eval_expression(value_expr, env.clone())?;
        pairs.insert(hash_key, HashPair { key, value });
    }
    Ok(Object::Hash(HashObject { pairs }))
}

fn eval_hash_index_expression(hash: HashObject, index: Object) -> Result<Object, String> {
    let hash_key = object_to_hash_key(&index)?;
    match hash.pairs.get(&hash_key) {
        Some(pair) => Ok(pair.value.clone()),
        None => Ok(Object::Null(Null {})),
    }
}

fn object_to_hash_key(obj: &Object) -> Result<HashKey, String> {
    match obj {
        Object::Integer(i) => Ok(i.hash_key()),
        Object::Boolean(b) => Ok(b.hash_key()),
        Object::MonString(s) => Ok(s.hash_key()),
        _ => Err(format!("unusable as hash key: {:?}", obj)),
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

        let env = Rc::new(RefCell::new(Environment::new()));
        eval(&program, env)
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

    #[test]
    fn test_eval_return() {
        struct TestCase {
            input: &'static str,
            // Some(n) => 期望整数结果; None => 期望报错（has_error=true 时不检查）
            expected: Option<i64>,
            has_error: bool,
        }

        let tests = vec![
            // 基本 return
            TestCase {
                input: "return 2;",
                expected: Some(2),
                has_error: false,
            },
            // return 后面的语句不执行
            TestCase {
                input: "return 2; 9",
                expected: Some(2),
                has_error: false,
            },
            // return 表达式
            TestCase {
                input: "return 1+2*3;",
                expected: Some(7),
                has_error: false,
            },
            // return 前面可以有其他语句
            TestCase {
                input: "9;return 1+2*3; 10",
                expected: Some(7),
                has_error: false,
            },
            // 缺分号 => 解析报错
            TestCase {
                input: "return 1",
                expected: None,
                has_error: true,
            },
            // if 块内 return，块外语句不执行
            TestCase {
                input: "if (10>1) {return 10; 1}",
                expected: Some(10),
                has_error: false,
            },
            // TODO: 嵌套 if + return 的场景目前还不能稳定解析，先跳过并保留用例
            // ★ 核心 case：嵌套 if，内层 return 10 必须穿透外层块
            // 如果 ReturnValue 冒泡实现有误，会错误地执行 return 1 得到 1
            TestCase {
                input: r#"if (10>1) {
                        if (10>1) {
                            return 10;
                        }
                        return 1;
                    }"#,
                expected: Some(10),
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
                    None => {} // has_error=false 但 expected=None 的情况目前没有
                }
            }
        }
    }

    #[test]
    fn test_let() -> Result<(), String> {
        struct TestCase {
            input: &'static str,
            expected: Option<i64>,
            has_error: bool,
        }

        let tests = vec![
            TestCase {
                input: "let a = 5; a",
                expected: Some(5),
                has_error: false,
            },
            TestCase {
                input: "let a = 6; a;",
                expected: Some(6),
                has_error: false,
            },
            TestCase {
                input: "let a = 5; let b = a; let c = (a + b) * 2; c",
                expected: Some(20),
                has_error: false,
            },
        ];

        for tc in &tests {
            let result = string_to_ast(tc.input)?;
            match tc.expected {
                Some(expected) => assert_integer_object(&result, expected),
                None => {}
            }
        }
        Ok(())
    }

    #[test]
    fn test_function_call() {
        struct TestCase {
            input: &'static str,
            expected: i64,
        }

        let tests = vec![
            TestCase {
                input: "let identity = fn(x) { x; }; identity(5);",
                expected: 5,
            },
            TestCase {
                input: "let add = fn(a, b) { a + b; }; add(2, 3);",
                expected: 5,
            },
            TestCase {
                input: "fn() { 42; }();",
                expected: 42,
            },
            TestCase {
                input: "let x = 10; let f = fn(y) { x + y; }; f(5);",
                expected: 15,
            },
        ];

        for tc in &tests {
            let result = string_to_ast(tc.input).expect(tc.input);
            assert_integer_object(&result, tc.expected);
        }
    }

    #[test]
    fn test_recursive_function_call() {
        let input = r#"
            let fact = fn(n) {
                if (n == 0) {
                    return 1;
                }
                return n * fact(n - 1);
            };
            fact(5);
        "#;
        let result = string_to_ast(input).expect(input);
        assert_integer_object(&result, 120);
    }

    #[test]
    fn test_recursive_map_iter() {
        let input = r#"
            let map = fn(arr, f) {
                let iter = fn(res, arr) {
                    if (len(arr) == 0) {
                        return res;
                    }
                    return iter(push(res, f(first(arr))), rest(arr));
                };

                iter([], arr);
            };

            let a = [1,2,3,4,5];
            let double = fn(x) { 2 * x };
            map(a, double);
        "#;

        let result = string_to_ast(input).expect(input);
        match result {
            Object::Array(arr) => {
                let expected = [2, 4, 6, 8, 10];
                assert_eq!(arr.len(), expected.len());
                for (idx, expected_value) in expected.iter().enumerate() {
                    let elem = arr.get(idx).expect("array index should exist");
                    match elem {
                        Object::Integer(i) => assert_eq!(i.value, *expected_value),
                        other => panic!("expected Integer object, got: {:?}", other),
                    }
                }
            }
            other => panic!("expected Array object, got: {:?}", other),
        }
    }

    #[test]
    fn test_function_call_wrong_arity() {
        let tests = vec![
            "let add = fn(a, b) { a + b; }; add(1);",
            "let add = fn(a, b) { a + b; }; add(1, 2, 3);",
        ];

        for input in &tests {
            let result = string_to_ast(input);
            assert!(result.is_err(), "expected arity error for input: {}", input);
            let err = result.unwrap_err();
            assert!(
                err.contains("wrong number of arguments"),
                "unexpected error for input='{}': {}",
                input,
                err
            );
        }
    }

    #[test]
    fn test_string_concat() {
        struct TestCase {
            input: &'static str,
            expected_string: Option<&'static str>,
            expected_bool: Option<bool>,
            expected_err_contains: Option<&'static str>,
        }

        let tests = vec![
            TestCase {
                input: "\"hello \"+\"world\"",
                expected_string: Some("hello world"),
                expected_bool: None,
                expected_err_contains: None,
            },
            TestCase {
                input: "\"hello \"-\"world\"",
                expected_string: None,
                expected_bool: None,
                expected_err_contains: Some("unsupported infix operator for strings"),
            },
            TestCase {
                input: "\"hello\" == \"world\"",
                expected_string: None,
                expected_bool: Some(false),
                expected_err_contains: None,
            },
            TestCase {
                input: "\"hello\" == \"hello\"",
                expected_string: None,
                expected_bool: Some(true),
                expected_err_contains: None,
            },
        ];

        for tc in &tests {
            let got = string_to_ast(tc.input);

            if let Some(expected_err) = tc.expected_err_contains {
                assert!(got.is_err(), "expected error for input: {}", tc.input);
                let err = got.unwrap_err();
                assert!(
                    err.contains(expected_err),
                    "input: {}, expected err contains: {}, actual: {}",
                    tc.input,
                    expected_err,
                    err
                );
                continue;
            }

            let got = got.expect(tc.input);
            if let Some(expected) = tc.expected_string {
                match got {
                    Object::MonString(s) => assert_eq!(s.value, expected, "input: {}", tc.input),
                    other => panic!("input: {}, expected string, got {:?}", tc.input, other),
                }
                continue;
            }

            if let Some(expected) = tc.expected_bool {
                match got {
                    Object::Boolean(b) => assert_eq!(b.value, expected, "input: {}", tc.input),
                    other => panic!("input: {}, expected bool, got {:?}", tc.input, other),
                }
            }
        }
    }

    #[test]
    fn test_string_literal() {
        struct TestCase {
            input: &'static str,
            expected: &'static str,
        }

        let tests = vec![
            TestCase {
                input: "\"hello world\"",
                expected: "hello world",
            },
            TestCase {
                input: "\"hello world",
                expected: "hello world",
            },
        ];

        for tc in &tests {
            let got = string_to_ast(tc.input).expect(tc.input);
            match got {
                Object::MonString(s) => assert_eq!(s.value, tc.expected, "input: {}", tc.input),
                other => panic!("input: {}, expected string, got {:?}", tc.input, other),
            }
        }
    }

    #[test]
    fn test_builtin_len() {
        struct TestCase {
            input: &'static str,
            expected_int: Option<i64>,
            expected_err_contains: Option<&'static str>,
        }

        let tests = vec![
            TestCase {
                input: r#"len("hello")"#,
                expected_int: Some(5),
                expected_err_contains: None,
            },
            TestCase {
                input: r#"len("")"#,
                expected_int: Some(0),
                expected_err_contains: None,
            },
            TestCase {
                input: r#"len("hello", "world")"#,
                expected_int: None,
                expected_err_contains: Some("wrong number of arguments"),
            },
            TestCase {
                input: "len(1)",
                expected_int: None,
                expected_err_contains: Some("not supported on INTEGER"),
            },
            TestCase {
                input: "len(true)",
                expected_int: None,
                expected_err_contains: Some("not supported on BOOLEAN"),
            },
        ];

        for tc in tests {
            let got = string_to_ast(tc.input);

            if let Some(expected_err) = tc.expected_err_contains {
                assert!(got.is_err(), "expected error for input: {}", tc.input);
                let err = got.unwrap_err();
                assert!(
                    err.contains(expected_err),
                    "input: {}, expected err contains: {}, got: {}",
                    tc.input,
                    expected_err,
                    err
                );
                continue;
            }

            let got = got.expect(tc.input);
            match got {
                Object::Integer(i) => {
                    assert_eq!(Some(i.value), tc.expected_int, "input: {}", tc.input)
                }
                other => panic!("input: {}, expected Integer, got {:?}", tc.input, other),
            }
        }
    }

    #[test]
    fn test_array() {
        struct TestCase {
            input: &'static str,
            expected: Vec<i64>,
        }

        let tests = vec![TestCase {
            input: "[1,2*3, 5+1]",
            expected: vec![1, 6, 6],
        }];

        for tc in tests {
            let got = string_to_ast(tc.input).expect(tc.input);
            match got {
                Object::Array(arr) => {
                    assert_eq!(arr.len(), tc.expected.len(), "input: {}", tc.input);
                    for (idx, expected) in tc.expected.iter().enumerate() {
                        let elem = arr.get(idx).unwrap_or_else(|| {
                            panic!("missing index {} for input {}", idx, tc.input)
                        });
                        match elem {
                            Object::Integer(i) => {
                                assert_eq!(
                                    i.value, *expected,
                                    "input: {}, index: {}",
                                    tc.input, idx
                                )
                            }
                            other => {
                                panic!("input: {}, expected Integer, got {:?}", tc.input, other)
                            }
                        }
                    }
                }
                other => panic!("input: {}, expected Array, got {:?}", tc.input, other),
            }
        }
    }

    #[test]
    fn test_index() {
        struct TestCase {
            input: &'static str,
            expected_int: Option<i64>,
            expected_err_contains: Option<&'static str>,
        }

        let tests = vec![
            TestCase {
                input: "[1,2,3][2]",
                expected_int: Some(3),
                expected_err_contains: None,
            },
            TestCase {
                input: "1[1]",
                expected_int: None,
                expected_err_contains: Some("is not indexable"),
            },
            TestCase {
                input: "[1,2,3][true]",
                expected_int: None,
                expected_err_contains: Some("the index should be an integer"),
            },
            TestCase {
                input: "[1,2,3][3]",
                expected_int: None,
                expected_err_contains: Some("out of bounds, len:3, visit:3"),
            },
            TestCase {
                input: "[1,2*3, 5+1][1]",
                expected_int: Some(6),
                expected_err_contains: None,
            },
            TestCase {
                input: "let a = [1,2,3,4,[5,6]]; a[4][1]",
                expected_int: Some(6),
                expected_err_contains: None,
            },
        ];

        for tc in tests {
            let got = string_to_ast(tc.input);
            if let Some(expected_err) = tc.expected_err_contains {
                assert!(got.is_err(), "expected error for input: {}", tc.input);
                let err = got.unwrap_err();
                assert!(
                    err.contains(expected_err),
                    "input: {}, expected err contains: {}, got: {}",
                    tc.input,
                    expected_err,
                    err
                );
                continue;
            }

            let got = got.expect(tc.input);
            match got {
                Object::Integer(i) => {
                    assert_eq!(Some(i.value), tc.expected_int, "input: {}", tc.input)
                }
                other => panic!("input: {}, expected Integer, got {:?}", tc.input, other),
            }
        }
    }

    #[test]
    fn test_hash_index() {
        struct TestCase {
            input: &'static str,
            expected_int: Option<i64>,
            expected_null: bool,
            expected_err_contains: Option<&'static str>,
        }

        let tests = vec![
            TestCase {
                input: r#"{"one": 1, "two": 2}["one"]"#,
                expected_int: Some(1),
                expected_null: false,
                expected_err_contains: None,
            },
            TestCase {
                input: r#"{"a": 1, 2: 3, true: 4}["a"]"#,
                expected_int: Some(1),
                expected_null: false,
                expected_err_contains: None,
            },
            TestCase {
                input: r#"{"a": 1, 2: 3, true: 4}[2]"#,
                expected_int: Some(3),
                expected_null: false,
                expected_err_contains: None,
            },
            TestCase {
                input: r#"{"a": 1, 2: 3, true: 4}[true]"#,
                expected_int: Some(4),
                expected_null: false,
                expected_err_contains: None,
            },
            TestCase {
                input: r#"{"a": 1}["missing"]"#,
                expected_int: None,
                expected_null: true,
                expected_err_contains: None,
            },
            TestCase {
                input: r#"{[1,2]: 3}"#,
                expected_int: None,
                expected_null: false,
                expected_err_contains: Some("unusable as hash key"),
            },
            TestCase {
                input: r#"{"a": 1}[fn(x){x}]"#,
                expected_int: None,
                expected_null: false,
                expected_err_contains: Some("unusable as hash key"),
            },
        ];

        for tc in tests {
            let got = string_to_ast(tc.input);
            if let Some(expected_err) = tc.expected_err_contains {
                assert!(got.is_err(), "expected error for input: {}", tc.input);
                let err = got.unwrap_err();
                assert!(
                    err.contains(expected_err),
                    "input: {}, expected err contains: {}, got: {}",
                    tc.input,
                    expected_err,
                    err
                );
                continue;
            }

            let got = got.expect(tc.input);
            if tc.expected_null {
                assert!(
                    matches!(got, Object::Null(_)),
                    "input: {}, expected Null, got {:?}",
                    tc.input,
                    got
                );
                continue;
            }

            match got {
                Object::Integer(i) => {
                    assert_eq!(Some(i.value), tc.expected_int, "input: {}", tc.input)
                }
                other => panic!("input: {}, expected Integer, got {:?}", tc.input, other),
            }
        }
    }
}
