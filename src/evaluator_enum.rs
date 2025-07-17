#![allow(dead_code)]

use crate::ast_enum as ast;
use crate::object_enum as object;
use crate::token;

const TRUE: object::Object = object::Object::Boolean(object::Boolean { value: true });
const FALSE: object::Object = object::Object::Boolean(object::Boolean { value: false });
const NULL: object::Object = object::Object::Null(object::Null {});

pub fn eval(node: ast::Node) -> Option<object::Object> {
    match node {
        ast::Node::Program(program) => eval_statements(program.statements),
        ast::Node::Statement(statement) => eval_statement(statement),
        ast::Node::Expression(expression) => eval_expression(expression),
    }
}

fn eval_statement(statement: ast::Statement) -> Option<object::Object> {
    match statement {
        ast::Statement::Expression(expr_stmt) => eval_expression(*expr_stmt.expression),
        ast::Statement::Block(block_stmt) => eval_statements(block_stmt.statements),
        _ => None,
    }
}

fn eval_statements(statements: Vec<ast::Statement>) -> Option<object::Object> {
    let mut result: Option<object::Object> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement));
    }
    result
}

#[rustfmt::skip]
fn eval_expression(expression: ast::Expression) -> Option<object::Object> {
    match expression {
        ast::Expression::IntegerLiteral(int_lit) => Some(object::Object::Integer(object::Integer {value: int_lit.value,})),
        ast::Expression::Boolean(boolean) => Some(if boolean.value {TRUE.clone()} else {FALSE.clone()}),
        ast::Expression::Prefix(prefix_expr) => {
            let right = eval(ast::Node::Expression(*prefix_expr.right)).unwrap();
            eval_prefix_expression(prefix_expr.operator, right)
        },
        ast::Expression::Infix(infix_expr) =>{
            let left = eval(ast::Node::Expression(*infix_expr.left)).unwrap();
            let right = eval(ast::Node::Expression(*infix_expr.right)).unwrap();
            return eval_infix_expression(infix_expr.operator, left, right);
        }
        ast::Expression::If(if_expr) => eval_if_expression(if_expr),
        ast::Expression::Identifier(ident) if ident.value == "null" => Some(NULL.clone()),
        _ => None,
    }
}

fn eval_if_expression(if_expr: ast::IfExpression) -> Option<object::Object> {
    let condition = eval(ast::Node::Expression(*if_expr.condition))?;

    if is_truthy(condition) {
        return eval_statement(ast::Statement::Block(if_expr.consequence));
    } else if if_expr.alternative.is_some() {
        return eval_statement(ast::Statement::Block(if_expr.alternative.unwrap()));
    } else {
        return Some(NULL.clone());
    }
}

fn is_truthy(c: object::Object) -> bool {
    match c {
        object::Object::Boolean(b) => b.value,
        object::Object::Null(_) => false,
        _ => true, // Any other type is considered truthy
    }
}

fn eval_prefix_expression(operator: String, right: object::Object) -> Option<object::Object> {
    match operator.as_str() {
        token::BANG => eval_bang_operator_expression(right),
        token::MINUS => eval_minus_prefix_operator_expression(right),
        _ => return Some(NULL.clone()),
    }
}

#[rustfmt::skip]
fn eval_infix_expression(operator: String, left: object::Object, right: object::Object ) -> Option<object::Object> {
    match (left, right) {
        //
        // Check if both left and right are Integer objects and handle arithmetic operations
        //
        (object::Object::Integer(left_int), object::Object::Integer(right_int)) => {
            eval_integer_infix_expression(operator, left_int, right_int)
        }

        //
        // Check if both left and right are Boolean objects and handle equality and inequality
        //
        (object::Object::Boolean(left_bool), object::Object::Boolean(right_bool)) => {
            match operator.as_str() {
                token::EQ => Some(object::Object::Boolean(object::Boolean { value: left_bool.value == right_bool.value })),
                token::NOT_EQ => Some(object::Object::Boolean(object::Boolean { value: left_bool.value != right_bool.value })),
                _ => Some(NULL.clone()),
            }
        }
        _ => Some(NULL.clone()),
    }
}

#[rustfmt::skip]
fn eval_integer_infix_expression(operator: String, left: object::Integer, right: object::Integer) -> Option<object::Object> {
    let left_int: i64 = left.value;
    let right_int: i64 = right.value;
    match operator.as_str() {
        token::PLUS => Some(object::Object::Integer(object::Integer{value: left_int + right_int})),
        token::MINUS => Some(object::Object::Integer(object::Integer{value: left_int - right_int})),
        token::ASTERISK => Some(object::Object::Integer(object::Integer{value: left_int * right_int})),
        token::SLASH => Some(object::Object::Integer(object::Integer{value: left_int / right_int})),

        token::LT => Some(object::Object::Boolean(object::Boolean{value: left_int < right_int})),
        token::GT => Some(object::Object::Boolean(object::Boolean{value: left_int > right_int})),
        token::EQ => Some(object::Object::Boolean(object::Boolean{value: left_int == right_int})),
        token::NOT_EQ => Some(object::Object::Boolean(object::Boolean{value: left_int != right_int})),
        _=>  Some(NULL.clone())
    }
}

#[rustfmt::skip]
fn eval_bang_operator_expression(right: object::Object) -> Option<object::Object> {
    match right {
        object::Object::Boolean(b) => Some(if b.value { FALSE.clone() } else { TRUE.clone() }),
        object::Object::Integer(i) => Some(if i.value == 0 { TRUE.clone() } else { FALSE.clone() }),
        object::Object::Null(_) => Some(TRUE.clone()),
    }
}

fn eval_minus_prefix_operator_expression(right: object::Object) -> Option<object::Object> {
    match right {
        object::Object::Integer(i) => {
            Some(object::Object::Integer(object::Integer { value: -i.value }))
        }
        _ => Some(NULL.clone()),
    }
}
#[cfg(test)]
mod tests {
    use crate::ast_enum::{self as ast, Program};
    use crate::evaluator_enum::eval;
    use crate::lexer;
    use crate::object_enum as object;
    use crate::parser;

    fn test_eval(input: &str) -> Option<object::Object> {
        let l: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut p: parser::Parser<'_> = parser::Parser::new(l);
        let program: Program = p.parse_program().unwrap();
        eval(ast::Node::Program(program))
    }

    fn test_integer_object(obj: object::Object, expected: i64) {
        match obj {
            object::Object::Integer(int_obj) => {
                assert_eq!(
                    int_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, int_obj.value
                );
            }
            _ => panic!("object is not Integer"),
        }
    }

    fn test_boolean_object(obj: object::Object, expected: bool) {
        match obj {
            object::Object::Boolean(bool_obj) => {
                assert_eq!(
                    bool_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, bool_obj.value
                );
            }
            _ => panic!("object is not Boolean"),
        }
    }

    fn test_null_object(obj: object::Object) {
        match obj {
            object::Object::Null(_) => (),
            _ => panic!("object is not NULL"),
        }
    }
    #[test]
    fn test_eval_integer_expression() {
        let tests: [(&'static str, i64); 2] = [("5", 5), ("10", 10)];

        for (input, expected) in tests.iter() {
            let evaluated: object::Object = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests: [(&'static str, bool); 19] = [
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
        let tests: [(&'static str, bool); 9] = [
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
        let tests: [(&'static str, i64); 16] = [
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
            let evaluated: object::Object = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected);
        }

        //
        // Test cases that should return null
        //
        for input in return_null_tests.iter() {
            let evaluated: object::Object = test_eval(input).unwrap();
            test_null_object(evaluated);
        }
    }
}
