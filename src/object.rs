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

// define error types
impl Error {
    pub fn type_mismatch(type_a: &str, type_b: &str) -> Self {
        Self {
            message: format!("Type mismatch: expected {}, got {}", type_a, type_b),
        }
    }

    pub fn bad_operator(operator: &str, type_a: &str, type_b: Option<&str>) -> Self {
        Self {
            message: match type_b {
                Some(type_b) => format!(
                    "Bad operator: {} between {} and {}",
                    operator, type_a, type_b
                ),
                None => format!("Bad operator: {} for {}", operator, type_a),
            },
        }
    }

    pub fn unknown_operator(operator: &str, type_a: &str, type_b: Option<&str>) -> Self {
        Self {
            message: match type_b {
                Some(type_b) => format!(
                    "Unknown operator: {} for {} and {}",
                    operator, type_a, type_b
                ),
                None => format!("Unknown operator: {} for {}", operator, type_a),
            },
        }
    }

    pub fn undefined_variable(name: &str) -> Self {
        Self {
            message: format!("Undefined variable: {}", name),
        }
    }

    pub fn custom(message: String) -> Self {
        Self { message }
    }
}
