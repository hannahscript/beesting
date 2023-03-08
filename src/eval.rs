use crate::errors::ReplError;
use crate::parser::{Ast, ParserError, UserFunction};
use crate::root_env::{lookup, Environment};
use std::collections::HashMap;
use std::iter::zip;
use std::ops::Deref;

pub fn eval(ast: &Ast, env: &mut Environment) -> Result<Ast, ReplError> {
    match ast {
        Ast::List(xs) => eval_list(xs, env),
        Ast::Symbol(s) => eval_symbol(s, env),
        Ast::Integer(n) => Ok(Ast::Integer(*n)),
        Ast::Boolean(b) => Ok(Ast::Boolean(*b)),
        Ast::Function(_) => Ok(Ast::Nil),
        Ast::Builtin(_, _) => Ok(Ast::Nil),
        Ast::Nil => Ok(Ast::Nil),
    }
}

fn eval_list(xs: &Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    if xs.is_empty() {
        todo!("error: empty list")
    }

    if let Ast::Symbol(s) = &xs[0] {
        match s.as_str() {
            "def!" => eval_form_def(xs, env),
            "let*" => eval_form_let(xs, env),
            "if" => eval_form_if(xs, env),
            "fun*" => eval_form_fun(xs),
            _ => eval_func_call(xs, env),
        }
    } else {
        todo!("no special form and no function in list head")
    }
}

fn eval_form_def(args: &Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    // todo arity check
    let name = get_symbol_name(&args[1])?;
    let definition = &args[2];

    let definition_value = eval(definition, env)?;
    let values = env.front_mut().expect("Empty environments");
    values.insert(name, definition_value.clone()); // todo do i clone here or below? probably here right
    Ok(definition_value)
}

fn eval_form_let(args: &Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let bindings = &args[1];
    let expr = &args[2];

    bind_let(bindings.clone(), env)?;
    let result = eval(expr, env);
    env.pop_front();
    result
}

fn eval_form_if(args: &Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let condition = eval(&args[1], env)?;

    let is_true = match condition {
        Ast::Boolean(b) => b,
        _ => true,
    };

    if is_true {
        eval(&args[2], env)
    } else {
        eval(&args[3], env)
    }
}

fn eval_form_fun(args: &Vec<Ast>) -> Result<Ast, ReplError> {
    let params = get_symbol_list(&args[1])?;
    let body = &args[2];
    let fun = Ast::Function(Box::new(UserFunction {
        params,
        body: body.clone(),
    }));
    Ok(fun)
}

fn eval_func_call(xs: &Vec<Ast>, env: &mut Environment) -> Result<Ast, ReplError> {
    let fun = eval(&xs[0], env)?;
    let args = eval_all(&xs[1..], env)?;

    match fun {
        Ast::Function(fun_box) => {
            let user_fun = fun_box.deref();
            bind_fn(&user_fun.params, args, env);
            let result = eval(&user_fun.body, env);
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

fn eval_symbol(s: &str, env: &mut Environment) -> Result<Ast, ReplError> {
    let v = lookup(s, env)?;
    Ok(v.clone())
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
        values.push(eval(&x, env)?);
    }
    Ok(values)
}

fn get_symbol_list(ast: &Ast) -> Result<Vec<String>, ReplError> {
    if let Ast::List(xs) = ast {
        let mut result = vec![];
        for x in xs {
            result.push(get_symbol_name(x)?);
        }

        Ok(result)
    } else {
        todo!("Error: not a list")
    }
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
            let v = eval(&x, env)?;
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
