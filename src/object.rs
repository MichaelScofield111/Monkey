use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Integer => write!(f, "INTEGER"),
            ObjectType::Boolean => write!(f, "BOOLEAN"),
            ObjectType::Null => write!(f, "NULL"),
            ObjectType::ReturnValue => write!(f, "RETURN_VALUE"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Null(Null),
    ReturnValue(ReturnValue),
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
            Object::ReturnValue(r) => r.inspect(),
        }
    }
    // ...
}

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: i64,
}

impl Integer {
    pub fn inspect(&self) -> String {
        format!("{}", self.value)
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::Integer
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub value: bool,
}

impl Boolean {
    pub fn inspect(&self) -> String {
        format!("{}", self.value)
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::Boolean
    }
}

#[derive(Debug, Clone)]
pub struct Null;

impl Null {
    pub fn inspect(&self) -> String {
        "null".to_string()
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::Null
    }
}

#[derive(Debug, Clone)]
pub struct ReturnValue {
    pub value: Box<Object>, // 必须用 Box，因为 Object 是递归类型
}

impl ReturnValue {
    pub fn inspect(&self) -> String {
        self.value.inspect()
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::ReturnValue // 返回自身类型，而不是内部值的类型
    }
}
