mod ast;
mod lexer;
mod parser;
mod repl;
mod token;
mod utils;
mod object;
mod evaluator;

const BANNER:&str = r"
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

