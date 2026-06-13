#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::object::Object;
pub type Env = Rc<RefCell<Environment>>;


#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String,  Object>,
    outer: Option<Env>,
}

impl Environment {
    pub fn new() -> Env {
        Rc::new(RefCell::new(Environment{
            store: HashMap::new(),
            outer: None,
        }))
    }

    pub fn new_enclosed(outer: Env) -> Env{
        Rc::new(RefCell::new(Environment{
            store: HashMap::new(),
            outer: Some(outer),
        }))
    }

    pub fn define(&mut self, name: String, value: Object){
        self.store.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Object>{
        match self.store.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(outer_env) = &self.outer {
                    outer_env.borrow().get(name)
                } else {
                    None
                }
            }
        }
    }
}
