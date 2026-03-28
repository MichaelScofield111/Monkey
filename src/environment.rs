use crate::object::Object;
use std::collections::HashMap;

pub struct Environment {
    vars: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: &str, value: Object) {
        self.vars.insert(name.to_string(), value);
    }
}
