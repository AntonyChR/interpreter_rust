pub const INTEGER_OBJ: &str = "INTEGER";
pub const BOOLEAN_OBJ: &str = "BOOLEAN";
pub const RETURN_OBJ: &str = "RETURN";
pub const NULL_OBJ: &str = "NULL";
pub const ERROR_OBJ: &str = "ERROR";

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Return(Return),
    Null(Null),
    Error(Error),
}

#[allow(dead_code)]
impl Object {
    pub fn object_type(&self) -> &str {
        match self {
            Object::Integer(_) => INTEGER_OBJ,
            Object::Boolean(_) => BOOLEAN_OBJ,
            Object::Return(_) => RETURN_OBJ,
            Object::Null(_) => NULL_OBJ,
            Object::Error(_) => ERROR_OBJ,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.value.to_string(),
            Object::Boolean(b) => b.value.to_string(),
            Object::Return(r) => format!("return {}", r.value.inspect()),
            Object::Null(_) => "null".to_string(),
            Object::Error(e) => format!("Error: {}", e.message),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub value: Box<Object>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
}
