mod errors;
mod eval;
mod parser;
mod root_env;

use crate::errors::ReplError;
use crate::eval::eval;
use crate::parser::Ast;
use crate::root_env::{create_root_env, Environment};
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

fn read() -> Result<Ast, ReplError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.parse()?)
}

fn rep(root_env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let input = read()?;
    eval(input, &Rc::clone(root_env))
}

fn main() {
    let root_env = Rc::new(RefCell::new(create_root_env()));

    loop {
        print!("user> ");
        io::stdout().flush().expect("Can't flush. Call Luigi");
        let output_result = rep(&root_env);
        match output_result {
            Ok(output) => println!("{:?}", output),
            Err(err) => eprintln!("Error occurred: {:?}", err),
        }
    }
}

// (def! fib (fun* (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))
// (define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))
// (defun fib (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2)))))

// (def! fibt (fun* (n a b) (if (< n 1) a (fibt (- n 1) b (+ a b))) ))

// (def! add (fun* (acc limit) (if (< acc limit) (add (+ acc 1) limit) acc)))
