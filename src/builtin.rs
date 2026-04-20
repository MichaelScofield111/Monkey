use crate::object::{Array, Boolean, Builtin, Integer, MonString, Object};

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => Some(Object::Builtin(Builtin::new(name, builtin_len))),
        "first" => Some(Object::Builtin(Builtin::new(name, builtin_first))),
        "last" => Some(Object::Builtin(Builtin::new(name, builtin_last))),
        "rest" => Some(Object::Builtin(Builtin::new(name, builtin_rest))),
        "push" => Some(Object::Builtin(Builtin::new(name, builtin_push))),
        _ => None,
    }
}

fn builtin_push(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err(format!(
            "wrong number of arguments, expected {}, but got {}",
            2,
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => {
            let mut elements = arr.elements.clone();
            elements.push(args[1].clone());
            Ok(Object::Array(Array { elements }))
        }
        other => Err(format!("not supported on {}", object_kind(&other))),
    }
}

fn builtin_rest(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!(
            "wrong number of arguments, expected {}, but got {}",
            1,
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => arr
            .elements
            .split_first()
            .map(|(_, rest)| {
                Object::Array(Array {
                    elements: rest.to_vec(),
                })
            })
            .ok_or_else(|| "array is empty".to_string()),
        other => Err(format!("not supported on {}", object_kind(other))),
    }
}

fn builtin_last(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!(
            "wrong number of arguments, expected {}, but got {}",
            1,
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => arr
            .elements
            .last()
            .cloned()
            .ok_or_else(|| "array is empty".to_string()),
        other => Err(format!("not supported on {}", object_kind(other))),
    }
}

fn builtin_first(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!(
            "wrong number of arguments, expected {}, but got {}",
            1,
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => arr
            .elements
            .first()
            .cloned()
            .ok_or_else(|| "array is empty".to_string()),
        other => Err(format!("not supported on {}", object_kind(other))),
    }
}

fn builtin_len(args: Vec<Object>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!(
            "wrong number of arguments, expected {}, but got {}",
            1,
            args.len()
        ));
    }

    // type need to support `len` function
    match &args[0] {
        Object::MonString(s) => Ok(Object::Integer(Integer {
            value: s.value.len() as i64,
        })),
        Object::Array(arr) => Ok(Object::Integer(Integer {
            value: arr.elements.len() as i64,
        })),
        other => Err(format!("not supported on {}", object_kind(other))),
    }
}

fn object_kind(obj: &Object) -> &'static str {
    match obj {
        Object::Integer(_) => "INTEGER",
        Object::Boolean(Boolean { .. }) => "BOOLEAN",
        Object::Null(Null) => "NULL",
        Object::ReturnValue(_) => "RETURN_VALUE",
        Object::Function(_) => "FUNCTION",
        Object::MonString(MonString { .. }) => "STRING",
        Object::Builtin(_) => "BUILTIN",
        Object::Array(_) => "ARRAY",
    }
}
