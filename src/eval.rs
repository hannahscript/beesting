use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
use crate::root_env::Environment;

fn lookup<'a>(symbol: &str, env: &'a Environment) -> Result<&'a Ast, ReplError> {
    match env.values.get(symbol) {
        None => {
            if env.parent.is_none() {
                Err(ReplError::SymbolUndefined(symbol.to_owned()))
            } else {
                lookup(symbol, env.parent.as_ref().unwrap().as_ref())
            }
        }
        Some(v) => Ok(v),
    }
}

fn call(value: &Ast) -> Result<Ast, ParserError> {
    match value {
        Ast::List(xs) => {
            let fun = xs.get(0).unwrap();
            match fun {
                Ast::Function(name, fun) => fun(name, &xs[1..]),
                _ => panic!("Attempted to call non-function"),
            }
        }
        _ => panic!("Attempted to call non-list"),
    }
}

fn eval_ast<'a>(program: Ast, env: &Environment) -> Result<Ast, ReplError> {
    match program {
        Ast::Symbol(s) => {
            let v = lookup(&s, env)?;
            Ok(v.clone())
        }
        Ast::List(xs) => {
            let mut values = vec![];
            for x in xs {
                values.push(eval(x, env)?);
            }
            Ok(Ast::List(values))
        }
        _ => Ok(program),
    }
}

pub fn eval<'a>(ast: Ast, env: &Environment) -> Result<Ast, ReplError> {
    match ast {
        Ast::List(_) => {
            let v = eval_ast(ast, env)?;
            Ok(call(&v)?)
        }
        _ => Ok(eval_ast(ast, env)?),
    }
}
