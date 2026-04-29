use std::collections::HashMap;
use std::fmt;
use std::hash::Hasher;
use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{BlockStatement, Identifier, Node},
    environment::Environment,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Function,
    MonString,
    Builtin,
    Array,
    Hash,
    HashPair,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Integer => write!(f, "INTEGER"),
            ObjectType::Boolean => write!(f, "BOOLEAN"),
            ObjectType::Null => write!(f, "NULL"),
            ObjectType::ReturnValue => write!(f, "RETURN_VALUE"),
            ObjectType::Function => write!(f, "FUNCTION"),
            ObjectType::MonString => write!(f, "STRING"),
            ObjectType::Builtin => write!(f, "BUILTIN"),
            ObjectType::Array => write!(f, "ARRAY"),
            ObjectType::Hash => write!(f, "HASH"),
            ObjectType::HashPair => write!(f, "HASH_PAIR"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Null(Null),
    ReturnValue(ReturnValue),
    Function(Function),
    MonString(MonString),
    Builtin(Builtin),
    Array(Array),
    Hash(HashObject),
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
            Object::ReturnValue(r) => r.inspect(),
            Object::Function(f) => f.inspect(),
            Object::MonString(f) => f.inspect(),
            Object::Builtin(f) => f.inspect(),
            Object::Array(f) => f.inspect(),
            Object::Hash(f) => f.inspect(),
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
    pub fn hash_key(&self) -> HashKey {
        HashKey {
            object_type: self.object_type(),
            value: self.value as u64,
        }
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
    pub fn hash_key(&self) -> HashKey {
        HashKey {
            object_type: self.object_type(),
            value: if self.value { 1 } else { 0 },
        }
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

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn inspect(&self) -> String {
        format!(
            "fn({}) {{ {} }}",
            self.params
                .iter()
                .map(|p| p.value.clone())
                .collect::<Vec<_>>()
                .join(", "),
            self.body.string()
        )
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::Function
    }
}

#[derive(Debug, Clone)]
pub struct MonString {
    pub value: String,
}

impl MonString {
    pub fn inspect(&self) -> String {
        format!("\"{}\"", self.value)
    }
    pub fn object_type(&self) -> ObjectType {
        ObjectType::MonString
    }
    pub fn hash_key(&self) -> HashKey {
        let mut hasher = FnvHasher::new();
        hasher.write(self.value.as_bytes());
        HashKey {
            object_type: self.object_type(),
            value: hasher.finish(),
        }
    }
}

pub type BuiltinFunction = fn(args: Vec<Object>) -> Result<Object, String>;

#[derive(Debug, Clone)]
pub struct Builtin {
    name: String,
    fun: BuiltinFunction,
}

impl Builtin {
    pub fn new(name: &str, fun: BuiltinFunction) -> Self {
        Self {
            name: name.to_string(),
            fun,
        }
    }

    pub fn call(&self, args: Vec<Object>) -> Result<Object, String> {
        (self.fun)(args)
    }
    pub fn inspect(&self) -> String {
        format!("{} is a builtin\n", self.name)
    }

    pub fn object_type(&self) -> ObjectType {
        ObjectType::Builtin
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Object>,
}

impl Array {
    pub fn new(elements: Vec<Object>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get(&self, index: usize) -> Option<Object> {
        self.elements.get(index).cloned()
    }

    pub fn inspect(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .iter()
                .map(|e| e.inspect())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn object_type(&self) -> ObjectType {
        ObjectType::Array
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashKey {
    pub object_type: ObjectType,
    pub value: u64,
}

#[derive(Debug, Clone)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}

#[derive(Debug, Clone)]
pub struct HashObject {
    pub pairs: HashMap<HashKey, HashPair>,
}

impl HashObject {
    pub fn inspect(&self) -> String {
        let mut kvs = Vec::with_capacity(self.pairs.len());
        for pair in self.pairs.values() {
            kvs.push(format!("{}:{}", pair.key.inspect(), pair.value.inspect()));
        }
        format!("{{{}}}", kvs.join(","))
    }

    pub fn object_type(&self) -> ObjectType {
        ObjectType::Hash
    }
}

#[derive(Default)]
struct FnvHasher(u64);

impl FnvHasher {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const FNV_PRIME: u64 = 0x100000001b3;
        for b in bytes {
            self.0 ^= u64::from(*b);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
}
