use crate::object::Object;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct Environment {
    vars: HashMap<String, Object>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(value) = self.vars.get(name) {
            return Some(value.clone());
        }

        // recursive to find
        self.parent.as_ref().and_then(|p| p.borrow().get(name))
    }

    pub fn set(&mut self, name: &str, value: Object) {
        self.vars.insert(name.to_string(), value);
    }
}
