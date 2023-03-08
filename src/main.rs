mod errors;
mod eval;
mod parser;
mod root_env;

use crate::errors::ReplError;
use crate::eval::eval;
use crate::parser::Ast;
use crate::root_env::{create_root_env, Environment};
use std::io;
use std::io::Write;

fn read() -> Result<Ast, ReplError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.parse()?)
}

fn rep(root_env: &mut Environment) -> Result<Ast, ReplError> {
    let input = read()?;
    eval(&input, root_env)
}

fn main() {
    let mut root_env = create_root_env();

    loop {
        print!("user> ");
        io::stdout().flush().expect("Can't flush. Call Luigi");
        let output_result = rep(&mut root_env);
        match output_result {
            Ok(output) => println!("{:?}", output),
            Err(err) => eprintln!("Error occurred: {:?}", err),
        }
    }
}

// (def! fib (fun* (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))
// (define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))
// (defun fib (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2)))))
