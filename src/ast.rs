#![allow(dead_code)]

use std::any::Any;

use crate::token;

pub trait Node {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

pub trait Statement: Node {
    fn statement_node(self);
    fn as_any(&self) -> &dyn Any;
    fn print_debug_info(&self);
}

pub trait Expression: Node {
    fn expression_node(&self);
    fn as_any(&self) -> &dyn Any;
}

pub type BoxedStatement = Box<dyn Statement>;
pub type BoxedExpression = Box<dyn Expression>;

// root node
pub struct Program {
    pub statements: Vec<BoxedStatement>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            String::new()
        }
    }
    fn string(&self) -> String {
        let mut a:String = String::new();
        for stmt in self.statements.iter(){
            println!("{}",stmt.string());
            a.push_str(stmt.string().as_str())
        }
        return a;
    }
}

impl Program {
    fn string(&self) -> String {
        let mut out = String::new();
        for stmt in self.statements.iter() {
            out.push_str(stmt.string().as_str());
        }
        out
    }
}

#[derive(Debug)]
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
    fn string(&self) -> String {
        self.value.clone()
    }
}
impl Expression for Identifier {
    fn expression_node(&self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// concrete statements
pub struct LetStatement {
    pub token: token::Token,
    pub name: Identifier,
    pub value: Identifier,
}

impl Statement for LetStatement {
    fn statement_node(self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn print_debug_info(&self) {
        println!("");
        println!("Token -> {:?}", self.token);
        println!("value -> {:?}", self.value);
        println!("name ->");
        println!("    name.value ->{}", self.name.value);
        println!("    name.token-> {:?}", self.name.token);
    }
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = format!("{} {} = ", self.token_literal(), self.name.string());
        if self.value.string() != "" {
            out = format!("{}{}", out, self.value.string());
        }
        out = out + ";";
        out
    }
}

// return statement
pub struct ReturnStatement {
    pub token: token::Token,
    pub return_value: Option<BoxedExpression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = format!("{} ", self.token_literal());
        if let Some(expression) = &self.return_value {
            out.push_str(&expression.string());
        }
        out = out + ";";
        out
    }
}

impl Statement for ReturnStatement {
    fn statement_node(self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn print_debug_info(&self) {
        println!("");
        println!("Token -> {:?}", self.token);
    }
}

// expression statement

pub struct ExpressionStatement {
    pub token: token::Token,
    pub expression: Option<BoxedExpression>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        if let Some(expression) = &self.expression {
            return expression.string();
        }
        return String::new();
    }
}

impl Statement for ExpressionStatement {
    fn statement_node(self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn print_debug_info(&self) {
        println!("");
        println!("Token -> {:?}", self.token);
    }
}

pub struct IntegerLiteral {
    pub token: token::Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        format!("{}", self.value)
    }
    fn string(&self) -> String {
        self.token_literal()
    }
}

impl Expression for IntegerLiteral {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn expression_node(&self) {}
}

pub struct PrefixExpression {
    pub token: token::Token,
    pub operator: String,
    pub right: Option<BoxedExpression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        match &self.right {
            Some(value) => format!("({}{})", self.operator, value.string()),
            None => panic!("PrefixExpression.Right is a None value"),
        }
    }
}

impl Expression for PrefixExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn expression_node(&self) {}
}

pub struct InfixExpression {
    pub token: token::Token,
    pub left: Option<BoxedExpression>,
    pub operator: String,
    pub right: Option<BoxedExpression>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let left_string = self.left.as_ref().map_or_else(
            || panic!("no expression in left node"),
            |expr| expr.string(),
        );
        let right_string = self.right.as_ref().map_or_else(
            || panic!("no expression in right node"),
            |expr| expr.string(),
        );
        format!("({} {} {})", left_string, self.operator, right_string)
    }
}

impl Expression for InfixExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn expression_node(&self) {}
}

mod tests {
    #[test]
    fn test_string_method_by_node_trait() {
        use crate::ast;
        use crate::token;

        let program: ast::Program = ast::Program {
            statements: vec![Box::new(ast::LetStatement {
                token: token::Token {
                    toke_type: token::LET.to_string(),
                    literal: "let".to_string(),
                },
                name: ast::Identifier {
                    token: token::Token {
                        toke_type: token::IDENT.to_string(),
                        literal: "myVar".to_string(),
                    },
                    value: "myVar".to_string(),
                },
                value: ast::Identifier {
                    token: token::Token {
                        toke_type: token::IDENT.to_string(),
                        literal: "anotherVar".to_string(),
                    },
                    value: "anotherVar".to_string(),
                },
            })],
        };

        let expected_string = "let myVar = anotherVar;";
        assert_eq!(
            program.string(),
            expected_string,
            "program.string() wrong. gor =\"{}\"",
            expected_string
        );
    }
}
