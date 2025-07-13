#![allow(dead_code)]

use std::any::Any;

const INTEGER_OBJ: &str = "INTEGER";
const BOOLEAN_OBJ: &str = "BOOLEAN";
const NULL_OBJ: &str = "NULL";

type ObjectType = String;
pub type BoxedObject = Box<dyn Object>;

pub trait Object:Any{
    fn object_type(&self)->ObjectType;
    fn inspect(&self)->String;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T:Any + 'static> AsAny for T{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Integer{
    pub value: i64,
}

impl Object for Integer {
    fn object_type(&self)->ObjectType {
        String::from(INTEGER_OBJ)
    }
    fn inspect(&self)->String {
        format!("{}", self.value)
    }
}

pub struct Boolean {
    pub value: bool,
}

impl Object for Boolean{
    fn object_type(&self)->ObjectType {
        String::from(BOOLEAN_OBJ)
    }
    fn inspect(&self)->String {
        format!("{}", self.value)
    }
}

pub struct NULL {}

impl Object for NULL {
    fn object_type(&self)->ObjectType {
        String::from("null")
    }
    fn inspect(&self)->String {
        String::from(NULL_OBJ)
    }
}

