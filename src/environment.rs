#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::object::Object;

pub type Env<'a> = Rc<RefCell<Environment<'a>>>;

#[derive(Debug, Clone)]
pub struct Environment<'a> {
    store: HashMap<&'a str, Object<'a>>,
    outer: Option<Env<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Env<'a> {
        Rc::new(RefCell::new(Environment {
            store: HashMap::new(),
            outer: None,
        }))
    }

    pub fn new_enclosed(outer: Env<'a>) -> Env<'a> {
        Rc::new(RefCell::new(Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }))
    }

    pub fn define(&mut self, name: &'a str, value: Object<'a>) {
        self.store.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Object<'a>> {
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
