use std::collections::HashMap;

use crate::object::Object;
pub struct Environment {
    store: HashMap<String, Object>,
    //outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            //outer: None,
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.store.get(name)
    }

    // Additional methods for managing the environment can be added here
}
