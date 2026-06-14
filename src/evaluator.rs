#![allow(dead_code)]

use crate::ast;
use crate::environment::Env;
use crate::object::{self as object, Error, Object};

pub fn eval<'a>(node: ast::Node<'a>, env: Env<'a>) -> Option<Object<'a>> {
    match node {
        ast::Node::Program(program) => eval_program(program.statements, env),
        ast::Node::Statement(statement) => eval_statement(statement, env),
        ast::Node::Expression(expression) => eval_expression(expression, env),
    }
}

fn eval_statement<'a>(statement: ast::Statement<'a>, env: Env<'a>) -> Option<Object<'a>> {
    match statement {
        ast::Statement::Expression(expr_stmt) => eval_expression(*expr_stmt.expression, env),
        ast::Statement::Block(block_stmt) => eval_block_statement(block_stmt.statements, env),
        ast::Statement::Return(return_stmt) => eval_return_stmt(return_stmt, env),
        ast::Statement::Let(let_stmt) => eval_let_statement(let_stmt, env),
    }
}

fn eval_let_statement<'a>(stmt: ast::LetStatement<'a>, env: Env<'a>) -> Option<Object<'a>> {
    let evaluated: Option<Object<'_>> = eval(ast::Node::Expression(stmt.value), env.clone());
    if let Some(result) = evaluated {
        if result.object_type() == object::ERROR_OBJ {
            return Some(result);
        }
        env.borrow_mut().define(stmt.name.value, result);
    }
    None
}

fn eval_program<'a>(statements: Vec<ast::Statement<'a>>, env: Env<'a>) -> Option<Object<'a>> {
    let mut result: Option<Object<'a>> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement), env.clone());

        if let Some(res) = result.as_ref() {
            if let Object::Return(return_obj) = res {
                return Some(*return_obj.value.clone());
            } else if let Object::Error(_) = res {
                return result;
            }
        }
    }
    result
}

fn eval_return_stmt<'a>(stmt: ast::ReturnStatement<'a>, env: Env<'a>) -> Option<Object<'a>> {
    if let Some(return_value) = stmt.return_value {
        if let Some(res) = eval_expression(*return_value, env) {
            if res.object_type() == object::ERROR_OBJ {
                return Some(res);
            }
            return Some(Object::Return(object::Return {
                value: Box::new(res),
            }));
        }
        return None;
    }
    None
}

fn eval_block_statement<'a>(statements: Vec<ast::Statement<'a>>, env: Env<'a>) -> Option<Object<'a>> {
    let mut result: Option<Object<'a>> = None;
    for statement in statements {
        result = eval(ast::Node::Statement(statement), env.clone());

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
fn eval_expression<'a>(expression: ast::Expression<'a>, env: Env<'a>) -> Option<Object<'a>> {
    match expression {
        ast::Expression::IntegerLiteral(int_lit) => Some(Object::Integer(object::Integer { value: int_lit.value })),
        ast::Expression::Boolean(boolean) => Some(Object::Boolean(object::Boolean { value: boolean.value })),
        ast::Expression::Prefix(prefix_expr) => {
            if let Some(right) = eval(ast::Node::Expression(*prefix_expr.right), env) {
                if right.object_type() == object::ERROR_OBJ {
                    return Some(right);
                }
                return eval_prefix_expression(prefix_expr.operator, right);
            }
            None
        }
        ast::Expression::Infix(infix_expr) => {
            let left = match eval(ast::Node::Expression(*infix_expr.left), env.clone()) {
                Some(v) => v,
                None => return None,
            };

            let right = match eval(ast::Node::Expression(*infix_expr.right), env) {
                Some(v) => v,
                None => return None,
            };

            if left.object_type() == object::ERROR_OBJ {
                return Some(left);
            }
            if right.object_type() == object::ERROR_OBJ {
                return Some(right);
            }
            eval_infix_expression(infix_expr.operator, left, right)
        }
        ast::Expression::If(if_expr) => eval_if_expression(if_expr, env),
        ast::Expression::Identifier(ident) => eval_identifier(ident, env),
        ast::Expression::FunctionLiteral(func) => {
            Some(Object::Function(object::Function {
                parameters: func.parameters,
                body: func.body,
                env: env.clone(),
            }))
        }
        ast::Expression::Call(call_exp) => {
            if let Some(func) = eval(ast::Node::Expression(*call_exp.function), env.clone()) {
                if func.object_type() == object::ERROR_OBJ {
                    return Some(func);
                }
                let args = eval_expressions(call_exp.arguments, env);
                if args.len() == 1 && args[0].object_type() == object::ERROR_OBJ {
                    return Some(args[0].clone());
                }
                return Some(apply_function(func, args));
            }
            None
        }
    }
}

fn apply_function<'a>(func: Object<'a>, args: Vec<Object<'a>>) -> Object<'a> {
    match func {
        Object::Function(f) => {
            let new_enclosed_env = crate::environment::Environment::new_enclosed(f.env.clone());

            // define function args as variables in the new closure
            for (i, param) in f.parameters.iter().enumerate() {
                new_enclosed_env
                    .borrow_mut()
                    .define(param.value, args[i].clone());
            }

            let evaluated = eval(
                ast::Node::Statement(ast::Statement::Block(f.body)),
                new_enclosed_env,
            )
            .expect("expected Object value, got None. Location: \"apply_function\"");

            if let Object::Return(r) = evaluated {
                return *r.value;
            }
            evaluated
        }
        _ => Object::Error(Error::custom(format!(
            "not a function: {}",
            func.object_type()
        ))),
    }
}

fn eval_expressions<'a>(exps: Vec<ast::Expression<'a>>, env: Env<'a>) -> Vec<Object<'a>> {
    let mut evaluated_expressions = Vec::new();
    for e in exps {
        let evaluated = eval(ast::Node::Expression(e), env.clone());
        if let Some(res) = evaluated {
            if res.object_type() == object::ERROR_OBJ {
                return vec![res];
            }
            evaluated_expressions.push(res);
        }
    }
    evaluated_expressions
}

fn eval_identifier<'a>(identifier: ast::Identifier<'a>, env: Env<'a>) -> Option<Object<'a>> {
    if let Some(value) = env.borrow().get(identifier.value) {
        return Some(value);
    }
    Some(Object::Error(Error::undefined_variable(identifier.value)))
}

fn eval_if_expression<'a>(if_expr: ast::IfExpression<'a>, env: Env<'a>) -> Option<Object<'a>> {
    let condition = eval(ast::Node::Expression(*if_expr.condition), env.clone())?;

    if condition.object_type() == object::ERROR_OBJ {
        return Some(condition);
    }

    if is_truthy(condition) {
        return eval_statement(ast::Statement::Block(if_expr.consequence), env);
    } else if let Some(alt) = if_expr.alternative {
        return eval_statement(ast::Statement::Block(alt), env);
    } else {
        return Some(Object::Null(object::Null {}));
    }
}

fn is_truthy(c: Object) -> bool {
    match c {
        Object::Boolean(b) => b.value,
        Object::Null(_) => false,
        _ => true, // Any other type is considered truthy
    }
}

fn eval_prefix_expression<'a>(operator: &'a str, right: Object<'a>) -> Option<Object<'a>> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => Some(Object::Error(Error::unknown_operator(
            operator,
            right.object_type(),
            None,
        ))),
    }
}

#[rustfmt::skip]
fn eval_infix_expression<'a>(operator: &'a str, left: Object<'a>, right: Object<'a>) -> Option<Object<'a>> {
    match (&left, &right) {
        (Object::Integer(left_int), Object::Integer(right_int)) => {
            eval_integer_infix_expression(operator, left_int, right_int)
        }
        (Object::Boolean(left_bool), Object::Boolean(right_bool)) => {
            match operator {
                "==" => Some(Object::Boolean(object::Boolean { value: left_bool.value == right_bool.value })),
                "!=" => Some(Object::Boolean(object::Boolean { value: left_bool.value != right_bool.value })),
                _ => Some(Object::Error(Error::bad_operator(
                    operator,
                    left.object_type(),
                    Some(right.object_type()),
                ))),
            }
        }
        _ => Some(Object::Error(Error::bad_operator(
            operator,
            left.object_type(),
            Some(right.object_type()),
        ))),
    }
}

#[rustfmt::skip]
fn eval_integer_infix_expression<'a>(operator: &str, left: &object::Integer, right: &object::Integer) -> Option<Object<'a>> {
    let left_int = left.value;
    let right_int = right.value;
    match operator {
        "+" => Some(Object::Integer(object::Integer{value: left_int + right_int})),
        "-" => Some(Object::Integer(object::Integer{value: left_int - right_int})),
        "*" => Some(Object::Integer(object::Integer{value: left_int * right_int})),
        "/" => Some(Object::Integer(object::Integer{value: left_int / right_int})),
        "<" => Some(Object::Boolean(object::Boolean{value: left_int < right_int})),
        ">" => Some(Object::Boolean(object::Boolean{value: left_int > right_int})),
        "==" => Some(Object::Boolean(object::Boolean{value: left_int == right_int})),
        "!=" => Some(Object::Boolean(object::Boolean{value: left_int != right_int})),
        _ => Some(Object::Error(Error::unknown_operator(
            operator,
            object::INTEGER_OBJ,
            Some(object::INTEGER_OBJ),
        ))),
    }
}

#[rustfmt::skip]
fn eval_bang_operator_expression<'a>(right: Object<'a>) -> Option<Object<'a>> {
    match right {
        Object::Boolean(b) => Some(Object::Boolean(object::Boolean { value: !b.value })),
        Object::Integer(i) => Some(Object::Boolean(object::Boolean { value: i.value == 0 })),
        Object::Null(_) => Some(Object::Boolean(object::Boolean { value: true })),
        _ => Some(Object::Boolean(object::Boolean { value: false })),
    }
}

fn eval_minus_prefix_operator_expression<'a>(right: Object<'a>) -> Option<Object<'a>> {
    if right.object_type() != object::INTEGER_OBJ {
        return Some(Object::Error(Error::bad_operator(
            "-",
            right.object_type(),
            None,
        )));
    }
    match right {
        Object::Integer(i) => Some(Object::Integer(object::Integer { value: -i.value })),
        _ => Some(Object::Null(object::Null {})),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{self as ast};
    use crate::evaluator::eval;
    use crate::object::Object;
    use crate::{environment, lexer, parser};

    fn test_eval(input: &str) -> Option<Object<'_>> {
        let l = lexer::Lexer::new(input);
        let mut p = parser::Parser::new(l);
        let program = p.parse_program().unwrap();
        let env = environment::Environment::new();
        eval(ast::Node::Program(program), env)
    }

    fn test_integer_object(obj: Object, expected: i64, case: &str) {
        match obj {
            Object::Integer(int_obj) => {
                assert_eq!(
                    int_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, int_obj.value
                );
            }
            _ => panic!(
                "object is not Integer\nfound\n{}\nfor case:\n{}",
                obj.inspect(),
                case
            ),
        }
    }

    fn test_boolean_object(obj: Object, expected: bool, case: &str) {
        match obj {
            Object::Boolean(bool_obj) => {
                assert_eq!(
                    bool_obj.value, expected,
                    "object has wrong value, expected {}, got {}",
                    expected, bool_obj.value
                );
            }
            _ => panic!(
                "object is not Boolean\nfound\n{}\nfor case: \n{}",
                obj.inspect(),
                case
            ),
        }
    }

    fn test_null_object(obj: Object, case: &str) {
        match obj {
            Object::Null(_) => (),
            _ => panic!(
                "object is not NULL\nfound:\n{}\nfor case:\n{}",
                obj.inspect(),
                case
            ),
        }
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = [
            ("5", 5),
            ("10", 10),
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
            test_integer_object(evaluated, *expected, input);
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
            test_boolean_object(evaluated, *expected, input);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = [
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
            test_boolean_object(evaluated, *expected, input);
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let return_int_tests = [
            ("if (true) { 10 }", 10),
            ("if (1) { 10 }", 10),
            ("if (1 < 2) { 10 }", 10),
            ("if (1 > 2) { 10 } else { 20 }", 20),
            ("if (1 < 2) { 10 } else { 20 }", 10),
        ];

        let return_null_tests = ["if (false) { 10 }", "if (1 > 2) { 10 }"];

        for (input, expected) in return_int_tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected, input);
        }

        for input in return_null_tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_null_object(evaluated, input);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = [
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            ("if (10 > 1) { if (10 > 1) { return 10; } return 1; }", 10),
        ];

        for (input, expected) in tests.iter() {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, *expected, input);
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = [
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];

        for (input, expected) in tests.iter() {
            let res = test_eval(input).expect("error evaluating input");
            test_integer_object(res, *expected, input);
        }
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x){x + 2;};";
        let evaluated = test_eval(input).expect("Error evaluating input");
        match evaluated {
            Object::Function(func) => {
                assert_eq!(1, func.parameters.len());
                assert_eq!("x", func.parameters[0].to_string());
                assert_eq!("(x + 2)", func.body.to_string());
            }
            _ => panic!("object is not a Function"),
        }
    }

    #[test]
    fn test_function_application() {
        let test = [
            ("let identity = fn(x) { x; }; identity(5);", 5),
            ("let identity = fn(x) { return x; }; identity(5);", 5),
            ("let double = fn(x) { x * 2; }; double(5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
            ("fn(x) { x; }(5)", 5),
        ];

        for (input, expected) in test.iter() {
            let res = test_eval(input).expect("error evaluating input");
            test_integer_object(res, *expected, input);
        }
    }
}
