use std::io::{stdin, stdout, Write};

use crate::ast;
use crate::environment::{Environment, Env};
use crate::evaluator;
use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;

const PROMPT: &str = ">> ";

pub fn start() {
    let env:Env  = Environment::new();
    let mut input: String = String::new();
    loop {
        print!("{}", PROMPT);
        stdout().flush().expect("failed to flush stdout");
        stdin()
            .read_line(&mut input)
            .expect("can not read user input");

        let lexer: Lexer = Lexer::new(&input);
        let mut parser: Parser = Parser::new(lexer);
        let program: ast::Program = parser.parse_program().expect("error parsing program");
        if parser.get_errors().len() != 0 {
            print_parser_errors(parser.get_errors());
            continue;
        }
        let evaluated: Option<Object> = evaluator::eval(ast::Node::Program(program),  env.clone());
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
