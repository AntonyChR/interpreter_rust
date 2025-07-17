#![allow(dead_code)]
use std::io::{stdin, stdout, Write};

use crate::ast_enum as ast;
//use crate::evaluator;
use crate::evaluator_enum as evaluator;
use crate::lexer;
use crate::parser;

const PROMPT: &str = ">> ";

pub fn start() {
    loop {
        print!("{}", PROMPT);
        stdout().flush().expect("failed to flush stdout");
        let mut input: String = String::new();
        stdin()
            .read_line(&mut input)
            .expect("can not read user input");
        let lexer: lexer::Lexer = lexer::Lexer::new(&input);
        let mut parser: parser::Parser = parser::Parser::new(lexer);
        let program: ast::Program = parser.parse_program().expect("error parsing program");
        if parser.get_errors().len() != 0 {
            print_parser_errors(parser.get_errors());
            continue;
        }
        let evaluated = evaluator::eval(ast::Node::Program(program));
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
