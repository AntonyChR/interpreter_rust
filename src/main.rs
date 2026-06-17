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
 |______|_| |_| |_|_| |_| |_|\__,_|";

const GIT_HASH: &str = env!("GIT_HASH");

fn main() {
    println!("{BANNER} {GIT_HASH}");
    repl::start();
}
