use crate::errors::ReplError;
use crate::parser::{Ast, ParserError, UserFunction};
use crate::root_env::{lookup, Environment};
use std::collections::HashMap;
use std::iter::zip;
use std::ops::Deref;

fn call_e(fun: Ast, args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    match fun {
        Ast::Function(fun_box) => {
            let user_fun = fun_box.deref();
            bind_fn(&user_fun.params, args, env);
            let result = eval(user_fun.body.clone(), env);
            env.pop_front();
            result
        }
        Ast::Builtin(params, cb) => {
            bind_fn(&params, args, env);
            let result = cb(env);
            env.pop_front();
            result
        }
        _ => todo!("error: attempted to call non-function"),
    }
}

fn bind_fn(params: &[String], args: Vec<Ast>, env: &mut Environment) {
    let mut values = HashMap::new();
    // todo check params length against args length
    for (name, ast) in zip(params, args) {
        values.insert(name.clone(), ast);
    }

    env.push_front(values);
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
                    "if" => do_if(xs, env),
                    "fun*" => do_fun(xs, env),
                    _ => do_call_e(xs, env),
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
    let result = eval(expr.clone(), env);
    env.pop_front();
    result
}

fn do_fun(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let params = get_symbol_list(args.get(1).unwrap())?;
    let body = args.get(2).unwrap();
    let fun = Ast::Function(Box::new(UserFunction {
        params,
        body: body.clone(),
    }));
    Ok(fun)
}

fn do_call_e(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let fun = eval(args.get(0).unwrap().clone(), env)?;
    let vs = eval_all(&args[1..], env)?;
    call_e(fun, vs, env)
}

fn do_if(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let condition = eval(args.get(1).unwrap().clone(), env)?;

    let is_true = match condition {
        Ast::Boolean(b) => b,
        _ => true,
    };

    if is_true {
        eval(args.get(2).unwrap().clone(), env)
    } else {
        eval(args.get(3).unwrap().clone(), env)
    }
}

// fn do_fun(args: Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
//     let symbols = get_symbol_list(args.get(1).unwrap())?;
// }

fn get_symbol_list(ast: &Ast) -> Result<Vec<String>, ReplError> {
    if let Ast::List(xs) = ast {
        let mut result = vec![];
        for x in xs {
            result.push(get_symbol_name(&x)?);
        }

        Ok(result)
    } else {
        todo!("Error: not a list")
    }
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
