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
        ast::Statement::Block(block_stmt) => eval_statements(block_stmt.statements),
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
            let right: Box<dyn Object> = eval(ast::Node::Expression(*prefix_expr.right)).unwrap();
            eval_prefix_expression(prefix_expr.operator, right)
        },
        ast::Expression::Infix(infix_expr) =>{
            let left: Box<dyn Object>= eval(ast::Node::Expression(*infix_expr.left)).unwrap();
            let right: Box<dyn Object> = eval(ast::Node::Expression(*infix_expr.right)).unwrap();
            return eval_infix_expression(infix_expr.operator, left, right);
        }
        ast::Expression::If(if_expr) => eval_if_expression(if_expr),
        ast::Expression::Identifier(ident) if ident.value == "null" => Some(Box::new(NULL)),
        _ => None,
    }
}

fn eval_if_expression(if_expr: ast::IfExpression) -> Option<Box<dyn Object>> {
    let condition: Box<dyn Object> = eval(ast::Node::Expression(*if_expr.condition))?;

    if is_truthy(condition) {
        return eval_statement(ast::Statement::Block(if_expr.consequence));
    } else if if_expr.alternative.is_some() {
        return eval_statement(ast::Statement::Block(if_expr.alternative.unwrap()));
    } else {
        return Some(Box::new(NULL));
    }
}

fn is_truthy(c: Box<dyn Object>) -> bool {
    match c.object_type().as_str() {
        object::BOOLEAN_OBJ => {
            let bool_obj: &object::Boolean = c.as_any().downcast_ref::<object::Boolean>().unwrap();
            bool_obj.value
        }
        object::NULL_OBJ => false,
        _ => true, // Any other type is considered truthy
    }
}

fn eval_prefix_expression(operator: String, right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    match operator.as_str() {
        token::BANG => eval_bang_operator_expression(right),
        token::MINUS => eval_minus_prefix_operator_expression(right),
        _ => return Some(Box::new(NULL)),
    }
}

#[rustfmt::skip]
fn eval_infix_expression(operator: String, left: Box<dyn Object>, right: Box<dyn Object> ) -> Option<Box<dyn Object>> {

    //
    // Check if both left and right are Integer objects and handle arithmetic operations
    //
    if left.object_type() == String::from(object::INTEGER_OBJ) && 
       left.object_type() == String::from(object::INTEGER_OBJ)
    {
        return eval_integer_infix_expression(operator, left, right);
    }

    //
    // Check if both left and right are Boolean objects and handle equality and inequality
    //
    if operator == token::EQ || operator == token::NOT_EQ{
        if left.object_type() == String::from(object::BOOLEAN_OBJ) &&
           left.object_type() == String::from(object::BOOLEAN_OBJ)
        {
            let left_bool: bool = left.as_any().downcast_ref::<object::Boolean>().unwrap().value;
            let right_bool: bool= right.as_any().downcast_ref::<object::Boolean>().unwrap().value;

            if  operator == token::EQ{
                return Some(Box::new(object::Boolean{value: left_bool == right_bool}))
            }
            if operator == token::NOT_EQ{
                return Some(Box::new(object::Boolean{value: left_bool != right_bool}))
            }
        }
    }

    Some(Box::new(NULL))
}

#[rustfmt::skip]
fn eval_integer_infix_expression(operator: String, left: Box<dyn Object>, right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    let left_int: i64 = left.as_any().downcast_ref::<object::Integer>()?.value;
    let right_int: i64 = right.as_any().downcast_ref::<object::Integer>()?.value;
    match operator.as_str() {
        token::PLUS => Some(Box::new(object::Integer{value: left_int + right_int})),
        token::MINUS => Some(Box::new(object::Integer{value: left_int - right_int})),
        token::ASTERISK => Some(Box::new(object::Integer{value: left_int * right_int})),
        token::SLASH => Some(Box::new(object::Integer{value: left_int / right_int})),

        token::LT => Some(Box::new(object::Boolean{value: left_int < right_int})),
        token::GT => Some(Box::new(object::Boolean{value: left_int > right_int})),
        token::EQ => Some(Box::new(object::Boolean{value: left_int == right_int})),
        token::NOT_EQ => Some(Box::new(object::Boolean{value: left_int != right_int})),
        _=>  Some(Box::new(NULL))
    }
}

#[rustfmt::skip]
fn eval_bang_operator_expression(right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    //
    // check if the right operand is a Boolean
    //
    let bool_obj: Option<&object::Boolean> = right.as_any().downcast_ref::<object::Boolean>();
    if let Some(bool_obj) = bool_obj {
        return Some(Box::new(object::Boolean {value: !bool_obj.value,}));
    }

    //
    // check if the right operand is a NULL
    //
    let null_obj: Option<&object::NULL> = right.as_any().downcast_ref::<object::NULL>();
    if null_obj.is_some() {
        return Some(Box::new(TRUE));
    }

    //
    // check if the right operand is an Integer
    // if it is 0, return TRUE, otherwise return FALSE
    //
    let int_obj: Option<&object::Integer> = right.as_any().downcast_ref::<object::Integer>();
    if let Some(int_obj) = int_obj {
        if int_obj.value == 0 {
            return Some(Box::new(TRUE));
        }
        return Some(Box::new(FALSE));
    }

    return Some(Box::new(FALSE));
}

fn eval_minus_prefix_operator_expression(right: Box<dyn Object>) -> Option<Box<dyn Object>> {
    //
    // check if the right operand is an Integer
    //
    let int_obj: Option<&object::Integer> = right.as_any().downcast_ref::<object::Integer>();
    if let Some(int_obj) = int_obj {
        return Some(Box::new(object::Integer {
            value: -int_obj.value,
        }));
    }

    // if not an Integer, return NULL
    Some(Box::new(NULL))
}
#[cfg(test)]
mod tests {
    use crate::ast_enum::{self as ast, Program};
    use crate::evaluator::eval;
    use crate::lexer;
    use crate::object::{self, Object};
    use crate::parser;

    fn test_eval(input: &str) -> Option<Box<dyn Object>> {
        let l: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut p: parser::Parser<'_> = parser::Parser::new(l);
        let program: Program = p.parse_program().unwrap();
        eval(ast::Node::Program(program))
    }

    fn test_integer_object(obj: Box<dyn Object>, expected: i64) {
        let result: Option<&object::Integer> = obj.as_any().downcast_ref::<object::Integer>();
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
        let result: Option<&object::Boolean> = obj.as_any().downcast_ref::<object::Boolean>();
        assert!(result.is_some(), "object is not object::Boolean");
        assert_eq!(
            result.unwrap().value,
            expected,
            "object has wrong value, expected {}, got {}",
            expected,
            result.unwrap().value
        );
    }

    fn test_null_object(obj: Box<dyn Object>) {
        let result: Option<&object::NULL> = obj.as_any().downcast_ref::<object::NULL>();
        assert!(result.is_some(), "object is not object::NULL");
    }
    #[test]
    fn test_eval_integer_expression() {
        let tests = [("5", 5), ("10", 10)];

        for (input, expected) in tests.iter() {
            let evaluated: Box<dyn Object> = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = [
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated, *expected);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_bang_operator() {
        let tests = [
            ("!true", false),
            ("!false", true),
            ("!!true", true),
            ("!!false", false),
            ("!null", true),
            ("!!null", false),
            ("!5", false),
            ("!!5", true),
            ("!0", true),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated, *expected);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_integer_expression() {
        let tests = [
            ("5", 5), 
            ("10", 10), 
            ("0", 0), 
            ("-5", -5), 
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
            ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_if_else_expressions() {
        let return_int_tests: [(&'static str, i64); 5] = [
            ("if (true) { 10 }", 10),
            ("if (1) { 10 }", 10),
            ("if (1 < 2) { 10 }", 10),
            ("if (1 > 2) { 10 } else { 20 }", 20),
            ("if (1 < 2) { 10 } else { 20 }", 10),
        ];

        let return_null_tests: [&'static str; 4] = [
            "if (false) { 10 }",              
            "if (1 > 2) { 10 }",
            "if (1 > 2) { 10 } else { null }",
            "if (1 < 2) { null } else { 20 }",
        ];

        //
        // Test cases that should return integers
        //
        for (input, expected) in return_int_tests.iter() {
            let evaluated: Box<dyn Object> = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }

        //
        // Test cases that should return null
        //
        for input in return_null_tests.iter() {
            let evaluated: Box<dyn Object> = test_eval(input).unwrap();
            test_null_object(evaluated);
        }
    }
}
