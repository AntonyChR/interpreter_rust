mod ast_enum;
mod environment;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;
mod utils;

mod evaluator_enum;
mod object_enum;

const BANNER: &str = r"
  ______                           
 |  ____|                          
 | |__   _ __ ___  _ __ ___   __ _ 
 |  __| | '_ ` _ \| '_ ` _ \ / _` |
 | |____| | | | | | | | | | | (_| |
 |______|_| |_| |_|_| |_| |_|\__,_| 0.1 
";

fn main() {
    println!("{BANNER}");
    repl::start();
}
