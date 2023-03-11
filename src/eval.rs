use crate::errors::ReplError;
use crate::parser::{Ast, ParserError, UserFunction};
use crate::root_env::{lookup, Environment};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;

enum EvalBehaviour {
    ReturnImmediately(Ast),
    LoopWithAst(Ast),
    LoopWithAstAndEnv(Ast, Environment),
}

pub fn eval(i_ast: Ast, i_env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let mut ast = i_ast;
    let mut env = Rc::clone(i_env);

    loop {
        match ast {
            Ast::List(xs) => {
                let behaviour = eval_list(xs, &env)?;
                match behaviour {
                    EvalBehaviour::ReturnImmediately(n_ast) => return Ok(n_ast),
                    EvalBehaviour::LoopWithAst(n_ast) => ast = n_ast,
                    EvalBehaviour::LoopWithAstAndEnv(n_ast, n_env) => {
                        ast = n_ast;
                        env = Rc::new(RefCell::new(n_env));
                    }
                }
            }
            Ast::Symbol(s) => return eval_symbol(s, &env),
            Ast::Integer(n) => return Ok(Ast::Integer(n)),
            Ast::Boolean(b) => return Ok(Ast::Boolean(b)),
            Ast::Function(_) => return Ok(Ast::Nil),
            Ast::Builtin(_, _) => return Ok(Ast::Nil),
            Ast::Nil => return Ok(Ast::Nil),
        }
    }
}

fn eval_list(xs: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<EvalBehaviour, ReplError> {
    if xs.is_empty() {
        todo!("error: empty list")
    }

    if let Ast::Symbol(s) = &xs[0] {
        match s.as_str() {
            "def!" => Ok(EvalBehaviour::ReturnImmediately(eval_form_def(xs, env)?)),
            "let*" => do_form_let(xs, env),
            "if" => Ok(EvalBehaviour::LoopWithAst(do_form_if(xs, env)?)),
            "fun*" => Ok(EvalBehaviour::ReturnImmediately(eval_form_fun(xs, env)?)),
            _ => eval_func_call(xs, env),
        }
    } else {
        todo!("no special form and no function in list head")
    }
}

fn eval_form_def(mut args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    // todo arity check
    let name = get_symbol_name(args.remove(1))?;
    let definition = args.remove(1);

    let definition_value = eval(definition, env)?;
    env.borrow_mut()
        .values
        .insert(name, definition_value.clone()); // todo do i clone here or below? probably here right
    Ok(definition_value)
}

fn do_form_let(
    mut args: Vec<Ast>,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalBehaviour, ReplError> {
    let expr = args.remove(2);
    let bindings = args.remove(1);

    let n_env = bind_let(bindings, env)?;
    Ok(EvalBehaviour::LoopWithAstAndEnv(expr, n_env))
}

fn do_form_if(mut args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let condition = eval(args.remove(1), env)?;

    let is_true = match condition {
        Ast::Boolean(b) => b,
        _ => true,
    };

    Ok(if is_true {
        args.remove(1)
    } else {
        args.remove(2)
    })
}

fn eval_form_fun(mut args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let params = get_symbol_list(args.remove(1))?;
    let fun = Ast::Function(Box::new(UserFunction {
        params,
        body: args.remove(1),
        env: Rc::clone(env),
    }));
    Ok(fun)
}

fn eval_func_call(
    mut xs: Vec<Ast>,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalBehaviour, ReplError> {
    let fun_ast = xs.remove(0);
    let fun = eval(fun_ast, env)?;
    let args = eval_all(xs, env)?;

    match fun {
        Ast::Function(fun_box) => {
            let user_fun = fun_box;
            // env = Rc::new(RefCell::new(bind_fn(&user_fun.params, args, &user_fun.env)));
            // ast = user_fun.body;
            Ok(EvalBehaviour::LoopWithAstAndEnv(
                user_fun.body,
                bind_fn(&user_fun.params, args, &user_fun.env),
            ))
        }
        Ast::Builtin(params, cb) => {
            let n_env = bind_fn(&params, args, env);
            let result = cb(&Rc::new(RefCell::new(n_env)))?;
            Ok(EvalBehaviour::ReturnImmediately(result))
        }
        _ => todo!("error: attempted to call non-function"),
    }
}

fn eval_symbol(s: String, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let v = lookup(s, env)?;
    Ok(v)
}

fn bind_fn(params: &[String], args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Environment {
    let mut values = HashMap::new();
    // todo check params length against args length
    for (name, ast) in zip(params, args) {
        values.insert(name.clone(), ast);
    }

    Environment {
        values,
        parent: Some(Rc::clone(env)),
    }
}

fn eval_all(xs: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<Vec<Ast>, ReplError> {
    let mut values = vec![];
    for x in xs {
        values.push(eval(x, env)?);
    }
    Ok(values)
}

fn get_symbol_list(ast: Ast) -> Result<Vec<String>, ReplError> {
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

fn bind_let(ast: Ast, env: &Rc<RefCell<Environment>>) -> Result<Environment, ReplError> {
    let mut values = HashMap::new();

    let xs = match ast {
        Ast::List(xs) => xs,
        _ => todo!("error"),
    };

    let mut symbol = "".to_owned();
    let mut get_sym = true;
    for x in xs {
        if get_sym {
            symbol = get_symbol_name(x)?;
            get_sym = false;
        } else {
            let v = eval(x, env)?;
            values.insert(symbol.clone(), v);
            get_sym = true;
        }
    }

    Ok(Environment {
        values,
        parent: Some(env.clone()),
    })
}

fn get_symbol_name(ast: Ast) -> Result<String, ReplError> {
    match ast {
        Ast::Symbol(s) => Ok(s),
        _ => Err(ReplError::ParserError(ParserError::ExpectedSymbol)),
    }
}
