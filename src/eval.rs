use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
use crate::root_env::Environment;
use std::collections::HashMap;

fn lookup<'a>(symbol: &str, envs: &'a Environment) -> Result<&'a Ast, ReplError> {
    for env in envs {
        if let Some(v) = env.get(symbol) {
            return Ok(v);
        }
    }

    Err(ReplError::SymbolUndefined(symbol.to_owned()))
}

fn call(fun: Ast, args: Vec<Ast>) -> Result<Ast, ParserError> {
    match fun {
        Ast::Function(name, fun_p) => fun_p(&name, &args),
        _ => panic!("Attempted to call non-function"),
    }
}

fn eval_all(xs: &[Ast], env: &mut Environment) -> Result<Vec<Ast>, ReplError> {
    let mut values = vec![];
    for x in xs {
        values.push(eval(x.clone(), env)?);
    }
    Ok(values)
}

pub fn eval(ast: Ast, env: &mut Environment) -> Result<Ast, ReplError> {
    match ast {
        Ast::List(xs) => {
            if xs.is_empty() {
                return Ok(Ast::List(vec![]));
            }

            let first = xs.get(0).unwrap();
            if let Ast::Symbol(s) = first {
                match s.as_str() {
                    "def!" => do_def(xs, env),
                    "let*" => do_let(xs, env),
                    _ => do_call(xs, env),
                }
            } else {
                todo!("no special form and no function in list head")
            }
        }
        Ast::Symbol(s) => {
            let v = lookup(&s, env)?;
            Ok(v.clone())
        }
        _ => Ok(ast),
    }
}

fn do_def(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    // todo arity check
    let name = get_symbol_name(args.get(1).unwrap())?;
    let definition = args.get(2).unwrap();

    let definition_value = eval(definition.clone(), env)?;
    bind(env, name, definition_value.clone());
    Ok(definition_value)
}

fn do_let(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let bindings = args.get(1).unwrap();
    let expr = args.get(2).unwrap();

    bind_let(bindings.clone(), env)?;
    eval(expr.clone(), env)
}

fn do_call(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let fun = eval(args.get(0).unwrap().clone(), env)?;
    let vs = eval_all(&args[1..], env)?;
    Ok(call(fun, vs)?)
}

fn bind(env: &mut Environment, name: String, ast: Ast) {
    let values = env.front_mut().expect("Empty environments");
    values.insert(name, ast);
}

fn bind_let(ast: Ast, env: &mut Environment) -> Result<(), ReplError> {
    let mut values = HashMap::new();

    let xs = match ast {
        Ast::List(xs) => xs,
        _ => todo!("error"),
    };

    let mut symbol = "".to_owned();
    let mut get_sym = true;
    for x in xs {
        if get_sym {
            symbol = get_symbol_name(&x)?;
            get_sym = false;
        } else {
            let v = eval(x, env)?;
            values.insert(symbol.clone(), v);
            get_sym = true;
        }
    }

    env.push_front(values);
    Ok(())
}

fn get_symbol_name(ast: &Ast) -> Result<String, ReplError> {
    match ast {
        Ast::Symbol(s) => Ok(s.clone()),
        _ => Err(ReplError::ParserError(ParserError::ExpectedSymbol)),
    }
}
