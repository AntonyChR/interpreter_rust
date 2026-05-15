#![allow(dead_code)]

use std::any::Any;

pub const INTEGER_OBJ: &str = "INTEGER";
pub const BOOLEAN_OBJ: &str = "BOOLEAN";
pub const NULL_OBJ: &str = "NULL";
pub const RETURN_VALUE_OBJ: &str = "RETURN_VALUE";

type ObjectType = String;

pub type BoxedObject = Box<dyn Object>;

pub trait Object: Any {
    fn object_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

pub struct Integer {
    pub value: i64,
}

impl Object for Integer {
    fn object_type(&self) -> ObjectType {
        String::from(INTEGER_OBJ)
    }
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Boolean {
    pub value: bool,
}

impl Object for Boolean {
    fn object_type(&self) -> ObjectType {
        String::from(BOOLEAN_OBJ)
    }
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NULL {}

impl Object for NULL {
    fn object_type(&self) -> ObjectType {
        String::from("null")
    }
    fn inspect(&self) -> String {
        String::from(NULL_OBJ)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ReturnValue {
    pub value: Box<dyn Object>,
}

impl Object for ReturnValue {
    fn object_type(&self) -> ObjectType {
        String::from(RETURN_VALUE_OBJ)
    }

    fn inspect(&self) -> String {
        self.value.inspect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}
