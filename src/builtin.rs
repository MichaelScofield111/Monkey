use crate::object::{Boolean, Builtin, Integer, MonString, Object};

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => Some(Object::Builtin(Builtin::new(name, builtin_len))),
        _ => None,
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

    match &args[0] {
        Object::MonString(s) => Ok(Object::Integer(Integer {
            value: s.value.len() as i64,
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
    }
}
