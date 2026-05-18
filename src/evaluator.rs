#![allow(dead_code)]

use crate::{ast, environment};
use crate::object::{self as object, Error, Object};
use crate::token;

const TRUE: Object = Object::Boolean(object::Boolean { value: true });
const FALSE: Object = Object::Boolean(object::Boolean { value: false });
const NULL: Object = Object::Null(object::Null {});

pub fn eval(node: ast::Node, env: &mut environment::Environment) -> Option<Object> {
    match node {
        ast::Node::Program(program) => eval_program(program.statements, env),
        ast::Node::Statement(statement) => eval_statement(statement, env),
        ast::Node::Expression(expression) => eval_expression(expression, env),
    }
}

fn eval_statement(statement: ast::Statement, env: &mut environment::Environment) -> Option<Object> {
    match statement {
        ast::Statement::Expression(expr_stmt) => eval_expression(*expr_stmt.expression, env),
        ast::Statement::Block(block_stmt) => eval_block_statement(block_stmt.statements, env),
        ast::Statement::Return(return_stmt) => eval_return_stmt(return_stmt, env),
        ast::Statement::Let(let_stmt) => eval_let_statement(let_stmt, env),
    }
}

fn eval_let_statement(stmt: ast::LetStatement, env: &mut environment::Environment) -> Option<Object> {
    let evaluated = eval(ast::Node::Expression(stmt.value), env);
    if let Some(result) = evaluated {
        if result.object_type() == object::ERROR_OBJ {
            return Some(result)
        }
        env.set(stmt.name.value, result);
    }
    None
}

fn eval_program(statements: Vec<ast::Statement>, env: &mut environment::Environment) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement), env);

        if result.is_some() {
            let result_clone: Object = result.clone().unwrap();
            if let Object::Return(return_obj) = result_clone {
                return Some(*return_obj.value);
            } else if let Object::Error(_) = result_clone {
                return result;
            }
        }
    }
    result
}

fn eval_return_stmt(stmt: ast::ReturnStatement, env: &mut environment::Environment) -> Option<Object> {
    if let Some(return_value) = stmt.return_value {
        if let Some(res) = eval_expression(*return_value,env) {

            if res.object_type() == object::ERROR_OBJ{
                return Some(res)
            }

            Some(Object::Return(object::Return {
                value: Box::new(res),
            }))

        } else {
            None
        }
    } else {
        None
    }
}

fn eval_block_statement(statements: Vec<ast::Statement>, env: &mut environment::Environment) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement), env);

        if let Some(res) = &result {
            let result_type = res.object_type();
            if result_type == object::RETURN_OBJ || result_type == object::ERROR_OBJ {
                return result;
            }
        }
    }
    result
}

#[rustfmt::skip]
fn  eval_expression(expression: ast::Expression, env: &mut environment::Environment) -> Option<Object> {
    match expression {
        ast::Expression::IntegerLiteral(int_lit) => Some(Object::Integer(object::Integer {value: int_lit.value,})),
        ast::Expression::Boolean(boolean) => Some(Object::Boolean(object::Boolean { value: boolean.value })),
        ast::Expression::Prefix(prefix_expr) => {
            if let Some(right) = eval(ast::Node::Expression(*prefix_expr.right), env){
                if right.object_type() == object::ERROR_OBJ {
                    return Some(right)
                }
                return eval_prefix_expression(prefix_expr.operator, right)
            }
            None
        },
        ast::Expression::Infix(infix_expr) =>{
            let left: Option<Object> = eval(ast::Node::Expression(*infix_expr.left), env);

            let right: Option<Object> = eval(ast::Node::Expression(*infix_expr.right), env);
            if left.is_none() || right.is_none() {
                return None;
            }
            if left.clone().unwrap().object_type() == object::ERROR_OBJ{
                return left;
             }
            if right.clone().unwrap().object_type() == object::ERROR_OBJ{
                return right;
             }
            return eval_infix_expression(infix_expr.operator, left.unwrap(), right.unwrap());
        }
        ast::Expression::If(if_expr) => eval_if_expression(if_expr, env),
        ast::Expression::Identifier(ident) => eval_identifier(ident, env),
        _ => None,
    }
}

fn eval_identifier(identifier: ast::Identifier, env: &mut environment::Environment) -> Option<Object>{
    if let Some(value) = env.get(identifier.value.as_str()) {
        return Some(value.clone())
    }
    return Some(Object::Error(Error::undefined_variable(identifier.value.as_str())))
}

fn eval_if_expression(if_expr: ast::IfExpression, env: &mut environment::Environment) -> Option<Object> {
    let condition = eval(ast::Node::Expression(*if_expr.condition), env)?;

    if condition.object_type() == object::ERROR_OBJ {
        return Some(condition);
    }

    if is_truthy(condition) {
        return eval_statement(ast::Statement::Block(if_expr.consequence), env);
    } else if if_expr.alternative.is_some() {
        return eval_statement(ast::Statement::Block(if_expr.alternative.unwrap()), env);
    } else {
        return Some(NULL.clone());
    }
}

fn is_truthy(c: Object) -> bool {
    match c {
        Object::Boolean(b) => b.value,
        Object::Null(_) => false,
        _ => true, // Any other type is considered truthy
    }
}

fn eval_prefix_expression(operator: String, right: Object) -> Option<Object> {
    match operator.as_str() {
        token::BANG => eval_bang_operator_expression(right),
        token::MINUS => eval_minus_prefix_operator_expression(right),
        _ => {
            return Some(Object::Error(Error::unknown_operator(
                operator.as_str(),
                right.object_type(),
                None,
            )))
        }
    }
}

#[rustfmt::skip]
fn eval_infix_expression(operator: String, left: Object, right: Object ) -> Option<Object> {
    let left_clone: Object = left.clone();
    let right_clone: Object = right.clone();
    let left_type: &str = left_clone.object_type();
    let right_type: &str = right_clone.object_type();
    
    match (left, right) {
        //
        // Check if both left and right are Integer objects and handle arithmetic operations
        //
        (Object::Integer(left_int), Object::Integer(right_int)) => {
            eval_integer_infix_expression(operator, left_int, right_int)
        }

        //
        // Check if both left and right are Boolean objects and handle equality and inequality
        //
        (Object::Boolean(left_bool), Object::Boolean(right_bool)) => {
            match operator.as_str() {
                token::EQ => Some(Object::Boolean(object::Boolean { value: left_bool.value == right_bool.value })),
                token::NOT_EQ => Some(Object::Boolean(object::Boolean { value: left_bool.value != right_bool.value })),
                _ => Some(Object::Error(Error::bad_operator(
                    operator.as_str(),
                    left_type,
                    Some(right_type),
                ))),
            }
        }
        _ => Some(Object::Error(Error::bad_operator(
            operator.as_str(),
            left_type,
            Some(right_type),
        ))),
    }
}

#[rustfmt::skip]
fn eval_integer_infix_expression(operator: String, left: object::Integer, right: object::Integer) -> Option<Object> {
    let left_int: i64 = left.value;
    let right_int: i64 = right.value;
    match operator.as_str() {
        token::PLUS => Some(Object::Integer(object::Integer{value: left_int + right_int})),
        token::MINUS => Some(Object::Integer(object::Integer{value: left_int - right_int})),
        token::ASTERISK => Some(Object::Integer(object::Integer{value: left_int * right_int})),
        token::SLASH => Some(Object::Integer(object::Integer{value: left_int / right_int})),

        token::LT => Some(Object::Boolean(object::Boolean{value: left_int < right_int})),
        token::GT => Some(Object::Boolean(object::Boolean{value: left_int > right_int})),
        token::EQ => Some(Object::Boolean(object::Boolean{value: left_int == right_int})),
        token::NOT_EQ => Some(Object::Boolean(object::Boolean{value: left_int != right_int})),
        _=>  Some(Object::Error(Error::unknown_operator(
            operator.as_str(),
            object::INTEGER_OBJ,
            Some(object::INTEGER_OBJ),
        ))),
    }
}

#[rustfmt::skip]
fn eval_bang_operator_expression(right: Object) -> Option<Object> {
    match right {
        Object::Boolean(b) => Some(if b.value { FALSE.clone() } else { TRUE.clone() }),
        Object::Integer(i) => Some(if i.value == 0 { TRUE.clone() } else { FALSE.clone() }),
        Object::Null(_) =>Some(TRUE.clone()),
        _ => Some(FALSE.clone()),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Option<Object> {
    if right.object_type() != object::INTEGER_OBJ {
        return Some(Object::Error(Error::bad_operator(
            token::MINUS,
            right.object_type(),
            None,
        )));
    }
    match right {
        Object::Integer(i) => Some(Object::Integer(object::Integer { value: -i.value })),
        _ => Some(NULL.clone()),
    }
}

fn new_error(message: String) -> object::Error {
    object::Error { message }
}

#[cfg(test)]
mod tests {
    use crate::ast::{self as ast, Program};
    use crate::evaluator::eval;
    use crate::object::{self as object, Error, Object};
    use crate::{environment, lexer, token};

    use crate::parser;

    fn test_eval(input: &str) -> Option<Object> {
        let l: lexer::Lexer<'_> = lexer::Lexer::new(input);
        let mut p: parser::Parser<'_> = parser::Parser::new(l);
        let program: Program = p.parse_program().unwrap();
        let mut env = environment::Environment::new();
        eval(ast::Node::Program(program), &mut env)
    }

    fn test_integer_object(obj: Object, expected: i64, case: String) {
        match obj {
            Object::Integer(int_obj) => {
                assert_eq!(
                    int_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, int_obj.value
                );
            }
            _ => panic!("object is not Integer\nfound\n{}\nfor case:\n{}", obj.inspect(), case),
        }
    }

    fn test_boolean_object(obj: Object, expected: bool, case: String) {
        match obj {
            Object::Boolean(bool_obj) => {
                assert_eq!(
                    bool_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, bool_obj.value
                );
            }
            _ => panic!("object is not Boolean\nfound\n{}\nfor case: \n{}",obj.inspect(), case),
        }
    }

    fn test_null_object(obj: Object, case: String) {
        match obj {
            Object::Null(_) => (),
            _ => panic!("object is not NULL\nfound:\n{}\nfor case:\n{}", obj.inspect(), case),
        }
    }
    #[rustfmt::skip]
    #[test]
    fn test_eval_integer_expression() {
        let tests: [(&'static str, i64); 15] = [
            ("5", 5), 
            ("10", 10), 
            ("-5",-5), 
            ("-10",-10),
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
            let evaluated: Object = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected, String::from(*input));
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
            test_boolean_object(evaluated, *expected, String::from(*input));
        }
    }

    #[rustfmt::skip]
    #[test]
    fn test_bang_operator() {
        let tests: [(&'static str, bool); 7] = [
            ("!true", false),
            ("!false", true),
            ("!!true", true),
            ("!!false", false),
            ("!5", false),
            ("!!5", true),
            ("!0", true),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_boolean_object(evaluated, *expected, String::from(*input));
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
            test_integer_object(evaluated, *expected, String::from(*input));
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

        let return_null_tests: [&'static str; 2] = [
            "if (false) { 10 }",              
            "if (1 > 2) { 10 }",
        ];

        //
        // Test cases that should return integers
        //
        for (input, expected) in return_int_tests.iter() {
            let evaluated: Object = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected, String::from(*input));
        }

        //
        // Test cases that should return null
        //
        for input in return_null_tests.iter() {
            let evaluated: Object = test_eval(input).unwrap();
            test_null_object(evaluated, String::from(*input));
        }
    }

    #[test]
    fn test_return_statements() {
        let tests: [(&'static str, i64); 5] = [
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            ("if (10 > 1) { if (10 > 1) { return 10; } return 1; }", 10),
        ];

        for (input, expected) in tests.iter() {
            let evaluated: Object = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected, String::from(*input));
        }
    }

    #[test]
    fn test_error_handling() {
        let tests: [(&'static str, Error); 9] = [
            (
                "5 + true;",
                Error::bad_operator(token::PLUS, object::INTEGER_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "5 + true; 5;",
                Error::bad_operator(token::PLUS, object::INTEGER_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "-true",
                Error::bad_operator(token::MINUS, object::BOOLEAN_OBJ, None),
            ),
            (
                " true + false;",
                Error::bad_operator(token::PLUS, object::BOOLEAN_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "5; true + false; 5",
                Error::bad_operator(token::PLUS, object::BOOLEAN_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "if (10 > 1) { true + false; }",
                Error::bad_operator(token::PLUS, object::BOOLEAN_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return true + false;
                    }
                    return 1;
                }
                ",
                Error::bad_operator(token::PLUS, object::BOOLEAN_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "if (true + false) { 10 }",
                Error::bad_operator(token::PLUS, object::BOOLEAN_OBJ, Some(object::BOOLEAN_OBJ)),
            ),
            (
                "foobar",
                Error::undefined_variable("foobar"),
            ),
        ];

        for (input, expected_error) in tests.iter() {
            let evaluated: Option<Object> = test_eval(input);
            assert!(
                evaluated.is_some(),
                "Expected an error, but got None, for input: {}",
                input
            );
            let evaluated_clone = evaluated.clone().unwrap();
            let object_type: String = String::from(evaluated_clone.object_type());
            let value: String = String::from(evaluated_clone.inspect());
            
            match evaluated.unwrap() {
                Object::Error(err) => {
                    assert_eq!(err.message, expected_error.message);
                }
                _ => panic!(
                    "Expected an error object, but got {:?}, for input: {}\n value: {}",
                    object_type, input, value
                ),
            }
        }
    }

    #[test]
    fn test_let_statements(){

        let tests: [(&'static str, i64); 4] = [
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];

        for c in tests.iter(){
            let res = test_eval(c.0);
            test_integer_object(res.unwrap(), c.1, String::from(c.0));            
        }

    }
}
