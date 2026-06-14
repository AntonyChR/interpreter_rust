use std::io::{stdin, stdout, Write};

use crate::ast;
use crate::environment::Environment;
use crate::evaluator;
use crate::lexer::Lexer;
use crate::parser::Parser;

const PROMPT: &str = ">> ";

pub fn start() {
    let env: std::rc::Rc<std::cell::RefCell<Environment<'_>>> = Environment::new();
    loop {
        print!("{}", PROMPT);
        stdout().flush().expect("failed to flush stdout");
        let mut input: String = String::new();
        stdin()
            .read_line(&mut input)
            .expect("can not read user input");

        if input.trim().is_empty() {
            continue;
        }

        let leaked_input: &'static str = Box::leak(input.into_boxed_str());

        let lexer: Lexer<'_> = Lexer::new(leaked_input);
        let mut parser = Parser::new(lexer);
        let program = match parser.parse_program() {
            Some(p) => p,
            None => {
                print_parser_errors(parser.get_errors());
                continue;
            }
        };

        if !parser.get_errors().is_empty() {
            print_parser_errors(parser.get_errors());
            continue;
        }

        let evaluated = evaluator::eval(ast::Node::Program(program), env.clone());
        if let Some(res) = evaluated {
            println!("{}", res.inspect());
        }
    }
}

fn print_parser_errors(errors: Vec<String>) {
    println!(" parser errors:");
    for msg in errors.iter() {
        println!("\t{}", msg);
    }
}
