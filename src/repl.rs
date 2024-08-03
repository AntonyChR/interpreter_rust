#![allow(dead_code)]
use std::io::{stdin, stdout, Write};

use crate::lexer;
use crate::token;

const PROMPT: &str = ">> ";

pub fn start() {
    loop {
        print!("{}", PROMPT);
        stdout().flush().expect("failed to flush stdout");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("can not read user input");
        let mut lexer: lexer::Lexer = lexer::Lexer::new(&input);
        let mut token: token::Token = lexer.next_token();
        while token.toke_type != token::EOF {
            println!(
                "token type:\"{}\", literal: \"{}\"",
                token.toke_type, token.literal
            );
            token = lexer.next_token();
        }
    }
}
