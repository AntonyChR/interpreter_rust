#![allow(dead_code)]

use crate::{ast, environment::Env};

pub const INTEGER_OBJ: &str = "INTEGER";
pub const BOOLEAN_OBJ: &str = "BOOLEAN";
pub const RETURN_OBJ: &str = "RETURN";
pub const NULL_OBJ: &str = "NULL";
pub const ERROR_OBJ: &str = "ERROR";
pub const FUNCTION_OBJ: &str = "FUNCTION";
pub const STRING_OBJ: &str = "STRING";

#[derive(Debug, Clone, PartialEq)]
pub enum Object<'a> {
    Integer(Integer),
    Boolean(Boolean),
    Return(Return<'a>),
    Null(Null),
    Error(Error),
    Function(Function<'a>),
    String_(String_),
}

#[allow(dead_code)]
impl<'a> Object<'a> {
    pub fn object_type(&self) -> &str {
        match self {
            Object::Integer(_) => INTEGER_OBJ,
            Object::Boolean(_) => BOOLEAN_OBJ,
            Object::Return(_) => RETURN_OBJ,
            Object::Null(_) => NULL_OBJ,
            Object::Error(_) => ERROR_OBJ,
            Object::Function(_) => FUNCTION_OBJ,
            Object::String_(_) => STRING_OBJ,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.value.to_string(),
            Object::Boolean(b) => b.value.to_string(),
            Object::String_(s) => s.value.clone(),
            Object::Return(r) => format!("return {}", r.value.inspect()),
            Object::Null(_) => "null".to_string(),
            Object::Error(e) => format!("Error: {}", e.message),
            Object::Function(f) => {
                let params: String = f
                    .parameters
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("fn({}) {{\n{}\n}}", params, f.body.to_string())
            }
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
pub struct String_{
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Null {}

#[derive(Debug, Clone, PartialEq)]
pub struct Return<'a> {
    pub value: Box<Object<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub parameters: Vec<ast::Identifier<'a>>,
    pub body: ast::BlockStatement<'a>,
    pub env: Env<'a>,
}

impl<'a> Clone for Function<'a> {
    fn clone(&self) -> Self {
        Self {
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            env: self.env.clone(),
        }
    }
}

impl<'a> PartialEq for Function<'a> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
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
