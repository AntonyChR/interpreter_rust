use std::io::{stdin, stdout, Write};

use crate::ast;
//use crate::environment::Environment;
use crate::evaluator;
use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;

const PROMPT: &str = ">> ";

pub fn start() {
    loop {
        print!("{}", PROMPT);
        stdout().flush().expect("failed to flush stdout");
        let mut input: String = String::new();
        stdin()
            .read_line(&mut input)
            .expect("can not read user input");

        //let mut env: Environment = Environment::new();
        let lexer: Lexer = Lexer::new(&input);
        let mut parser: Parser = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().expect("error parsing program");
        if parser.get_errors().len() != 0 {
            print_parser_errors(parser.get_errors());
            continue;
        }
        let evaluated: Option<Object> = evaluator::eval(ast::Node::Program(program));
        match evaluated {
            Some(obj) => {
                println!("{}", obj.inspect());
            }
            None => {
                println!("No evaluation result");
            }
        }
    }
}

fn print_parser_errors(errors: Vec<String>) {
    println!(" parser errors:");
    for msg in errors.iter() {
        println!("\t{}", msg);
    }
}
