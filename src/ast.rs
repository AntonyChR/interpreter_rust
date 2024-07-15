#![allow(dead_code)]

use std::any::Any;

use crate::{repl, token};

pub trait Node {
    /// only for debuging and testing
    fn token_literal(&self) -> String;
}

pub trait Statement: Node {
    fn statement_node(self);
    fn as_any(&self) -> &dyn Any;
    fn print_debug_info(&self);
}

pub trait Expression: Node {
    fn expression_node(self);
}

pub type TStatement = Box<dyn Statement>;

// root node
pub struct Program {
    pub statements: Vec<TStatement>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            String::new()
        }
    }
}

pub struct Identifier {
    pub token: token::Token,
    pub value: String,
}

impl Identifier {
    pub fn new_empty() -> Self {
        Self {
            token: token::Token::new_empty(),
            value: String::new(),
        }
    }
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl Expression for Identifier {
    fn expression_node(self) {}
}

// concrete statements
pub struct LetStatement {
    pub token: token::Token,
    pub name: Identifier,
    pub value: String,
}

impl LetStatement {}

impl Statement for LetStatement {
    fn statement_node(self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn print_debug_info(&self) {
        println!("");
        println!("Token -> {:?}", self.token);
        println!("value -> {}", self.value);
        println!("name ->");
        println!("    name.value ->{}", self.name.value);
        println!("    name.token-> {:?}", self.name.token);
    }
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

