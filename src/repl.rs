#![allow(dead_code)]

use std::io::{stdin, stdout, Write};

use crate::token::*;
use crate::lexer::*;

const PROMPT:&str = ">> ";

pub fn start(){
    loop {
        print!("{}",PROMPT);
        stdout().flush().expect("failed to flush stdout");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("can not read user input");
        let mut l = Lexer::new(&input);
        let mut tok:Token = l.next_token();
        while tok.type_f != EOF{
            println!("token type:\"{}\", literal: \"{}\"",tok.type_f, tok.literal);
            tok = l.next_token();
        }
    }
}


