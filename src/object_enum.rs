const INTEGER_OBJ: &str = "INTEGER";
const BOOLEAN_OBJ: &str = "BOOLEAN";
const NULL_OBJ: &str = "NULL";

type ObjectType = String;

pub enum ObjectEnum {
    Integer(Integer),
    Boolean(Boolean),
    Null(Null),
}

impl Object {
    pub fn object_type(&self) -> &str {
        match self {
            Object::Integer(_) => INTEGER_OBJ,
            Object::Boolean(_) => BOOLEAN_OBJ,
            Object::Null => NULL_OBJ,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.value.to_string(),
            Object::Boolean(b) => b.value.to_string(),
            Object::Null => NULL_OBJ.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Null {}
