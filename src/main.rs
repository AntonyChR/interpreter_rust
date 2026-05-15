mod ast;
mod environment;
mod lexer;
mod parser;
mod repl;
mod token;
mod utils;

mod evaluator;
mod object;

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
