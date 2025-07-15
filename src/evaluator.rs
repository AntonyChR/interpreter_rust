#![allow(dead_code)]

use crate::ast_enum as ast;
use crate::object::{self, Object};
use crate::token;

const TRUE: object::Boolean = object::Boolean { value: true };
const FALSE: object::Boolean = object::Boolean { value: false };
const NULL: object::NULL = object::NULL {};

pub fn eval(node: ast::Node) -> Option<Box<dyn Object>> {
    match node {
        ast::Node::Program(program) => eval_statements(program.statements),
        ast::Node::Statement(statement) => eval_statement(statement),
        ast::Node::Expression(expression) => eval_expression(expression),
    }
}
fn eval_statement(statement: ast::Statement) -> Option<Box<dyn Object>> {
    match statement {
        ast::Statement::Expression(expr_stmt) => eval_expression(*expr_stmt.expression),
        _ => None, // Manejar otros tipos de Statement si es necesario
    }
}

fn eval_statements(statements: Vec<ast::Statement>) -> Option<Box<dyn Object>> {
    let mut result: Option<Box<dyn Object>> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement));
    }
    result
}

#[rustfmt::skip]
fn eval_expression(expression: ast::Expression) -> Option<Box<dyn Object>> {
    match expression {
        ast::Expression::IntegerLiteral(int_lit) => Some(Box::new(object::Integer {value: int_lit.value,})),
        ast::Expression::Boolean(boolean) => Some(if boolean.value {Box::new(TRUE)} else {Box::new(FALSE)}),
        ast::Expression::Prefix(prefix_expr) => {
            let right = eval(ast::Node::Expression(*prefix_expr.right)).unwrap();
            eval_prefix_expression(prefix_expr.operator, right)
        },
        ast::Expression::Identifier(ident) if ident.value == "null" => Some(Box::new(NULL)),
        _ => None,
    }
}

fn eval_prefix_expression(operator: String, right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    match operator.as_str() {
        token::BANG => eval_bang_operator_expression(right),
        _ => return Some(Box::new(NULL)),
    }
}

#[rustfmt::skip]
fn eval_bang_operator_expression(right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    println!("Evaluating bang operator on: {}", right.inspect());
    let bool_obj: Option<&object::Boolean> = right.as_any().downcast_ref::<object::Boolean>();
    if let Some(bool_obj) = bool_obj {
        return Some(Box::new(object::Boolean {value: !bool_obj.value,}));
    }

    let null_obj: Option<&object::NULL> = right.as_any().downcast_ref::<object::NULL>();
    if null_obj.is_some() {
        return Some(Box::new(TRUE));
    }

    let int_obj: Option<&object::Integer> = right.as_any().downcast_ref::<object::Integer>();
    if let Some(int_obj) = int_obj {
        if int_obj.value == 0 {
            return Some(Box::new(TRUE));
        }
        return Some(Box::new(FALSE));
    }

    return Some(Box::new(FALSE));
}

#[cfg(test)]
mod tests {
    use crate::object::{self, Object};

    use crate::ast_enum as ast;
    use crate::evaluator::eval;
    use crate::lexer;
    use crate::parser;

    fn test_eval(input: &str) -> Option<Box<dyn Object>> {
        let l = lexer::Lexer::new(input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program().unwrap();
        println!("---------------- parsed program -------------");
        eval(ast::Node::Program(program))
    }

    fn test_integer_object(obj: Box<dyn Object>, expected: i64) {
        let result = obj.as_any().downcast_ref::<object::Integer>();
        assert!(result.is_some(), "object is not Integer");
        assert_eq!(
            result.unwrap().value,
            expected,
            "object has wrong value, expected {}, got {}",
            expected,
            result.unwrap().value
        );
    }

    fn test_boolean_object(obj: Box<dyn Object>, expected: bool) {
        let result = obj.as_any().downcast_ref::<object::Boolean>();
        assert!(result.is_some(), "object is not Boolean");
        assert_eq!(
            result.unwrap().value,
            expected,
            "object has wrong value, expected {}, got {}",
            expected,
            result.unwrap().value
        );
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = [("5", 5), ("10", 10)];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = [("true", true), ("false", false)];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated, *expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = [
            // ("!true", false),
            // ("!false", true),
            // ("!!true", true),
            // ("!!false", false),
            // ("!null", true),
            ("!5", false),
            ("!!5", true),
            ("!0", true),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated, *expected);
        }
    }
}
