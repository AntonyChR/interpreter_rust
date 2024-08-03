mod ast;
mod lexer;
mod parser;
mod repl;
mod token;
mod utils;
fn main() {
    let a = r"
  ______                           
 |  ____|                          
 | |__   _ __ ___  _ __ ___   __ _ 
 |  __| | '_ ` _ \| '_ ` _ \ / _` |
 | |____| | | | | | | | | | | (_| |
 |______|_| |_| |_|_| |_| |_|\__,_| 1.0 
";
  repl::start();
}

