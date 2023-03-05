mod errors;
mod eval;
mod parser;
mod root_env;

use crate::errors::ReplError;
use crate::eval::eval;
use crate::parser::Ast;
use crate::root_env::{create_root_env, Environment};
use std::io;

fn read() -> Result<Ast, ReplError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.parse()?)
}

fn rep(root_env: &Environment) -> Result<Ast, ReplError> {
    let input = read()?;
    eval(input, root_env)
}

fn main() {
    let root_env = create_root_env();

    loop {
        print!("user> ");
        let output_result = rep(&root_env);
        match output_result {
            Ok(output) => println!("{:?}", output),
            Err(err) => eprintln!("Error occurred: {:?}", err),
        }
    }
}
