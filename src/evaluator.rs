#![allow(dead_code)]

use crate::ast_enum as ast;
use crate::object::{self, Object};

const TRUE: object::Boolean = object::Boolean { value: true };
const FALSE: object::Boolean = object::Boolean { value: false };
const NULL: object::NULL = object::NULL {};

pub fn eval(node: ast::Node) -> Option<Box<dyn Object>> {
    match node {
        //ast::Node::Program(program) => eval_program(program),
        ast::Node::Statement(statement) => eval_statement(statement),
        ast::Node::Expression(expression) => eval_expression(expression),
    }
}

fn eval_program(program: ast::Program) -> Option<Box<dyn Object>> {
    let mut result: Option<Box<dyn Object>> = None;
    for statement in program.statements {
        result = eval(ast::Node::Statement(statement));
        // TODO: Handle return statements and errors to stop execution early.
    }
    result
}

fn eval_statement(statement: ast::Statement) -> Option<Box<dyn Object>> {
    match statement {
        ast::Statement::Expression(expr_stmt) => eval(ast::Node::Expression(*expr_stmt.expression)),
        _ => None, 
    }
}

fn eval_expression(expression: ast::Expression) -> Option<Box<dyn Object>> {
    match expression {
        ast::Expression::IntegerLiteral(int_lit) => Some(Box::new(object::Integer { value: int_lit.value })),
        ast::Expression::Boolean(boolean) => Some(Box::new(if boolean.value { TRUE } else { FALSE })),
        _ => None, 
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn test_eval(input: &str) -> Option<Box<dyn Object>> {
        let l = lexer::Lexer::new(input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program().unwrap();
        eval(ast::Node::Program(program))
    }

    fn test_integer_object(obj: &dyn Object, expected: i64) {
        let result = obj.as_any().downcast_ref::<object::Integer>();
        assert!(result.is_some(), "object is not Integer");
        assert_eq!(result.unwrap().value, expected, "object has wrong value");
    }
    
    fn test_boolean_object(obj: &dyn Object, expected: bool) {
        let result = obj.as_any().downcast_ref::<object::Boolean>();
        assert!(result.is_some(), "object is not Boolean");
        assert_eq!(result.unwrap().value, expected, "object has wrong value");
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = [
            ("5", 5),
            ("10", 10),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated.as_ref(), *expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = [
            ("true", true),
            ("false", false),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated.as_ref(), *expected);
        }
    }
}*/