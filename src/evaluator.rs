#![allow(dead_code)]

use crate::ast;
use crate::object;

pub enum ASTNode {
    IntegerLiteral(ast::IntegerLiteral),
    Program(ast::Program),
    ExpressionStatement(ast::ExpressionStatement),
}

pub fn eval(node: &dyn ast::Node) -> Option<object::BoxedObject> {
    
 return None

}

fn eval_statements(stmts: &[ast::BoxedStatement]) -> Option<object::BoxedObject> {
    let mut result: Option<object::BoxedObject> = None;

    for stmt in stmts {
        result 
    }

    result
}

mod tests {

    use crate::ast;
    use crate::evaluator;
    use crate::lexer;
    use crate::object;
    use crate::object::AsAny;
    use crate::parser;

    fn test_eval(input: &str) -> object::BoxedObject {
        let l: lexer::Lexer = lexer::Lexer::new(input);
        let mut p: parser::Parser = parser::Parser::new(l);
        let program: ast::Program = p.parse_program().unwrap();
        return evaluator::eval(&program as &dyn ast::Node).unwrap();
    }

    fn test_integer_object(obj: object::BoxedObject, expected: i64) {
        let result = match obj.as_any().downcast_ref::<object::Integer>() {
            Some(res) => res,
            None => panic!("object is not object::Integer"),
        };

        assert_eq!(
            result.value, expected,
            "object hast wrong value, expected={}, got={}",
            expected, result.value
        );
    }

    #[test]
    fn test_eval_integer_expression() {
        struct TC<'a> {
            input: &'a str,
            expected: i64,
        }

        let test_cases: [TC; 2] = [
            TC {
                input: "5",
                expected: 5,
            },
            TC {
                input: "10",
                expected: 10,
            },
        ];

        for tc in test_cases.iter() {
            let evaluated: object::BoxedObject = test_eval(tc.input);
            test_integer_object(evaluated, tc.expected);
        }
    }
}
