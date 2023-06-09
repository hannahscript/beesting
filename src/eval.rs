use crate::errors::ReplError;
use crate::parser::{Ast, ParserError, UserFunction};
use crate::root_env::{get_root, lookup, Environment};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;

enum EvalBehaviour {
    ReturnImmediately(Ast),
    LoopWithAst(Ast),
    LoopWithAstAndEnv(Ast, Rc<RefCell<Environment>>),
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
                        env = n_env;
                    }
                }
            }
            Ast::Symbol(s) => return eval_symbol(s, &env),
            Ast::Integer(n) => return Ok(Ast::Integer(n)),
            Ast::Boolean(b) => return Ok(Ast::Boolean(b)),
            Ast::String(str) => return Ok(Ast::String(str)),
            Ast::Function(f) => return Ok(Ast::Function(f)),
            Ast::Builtin(n, f) => return Ok(Ast::Builtin(n, f)),
            Ast::Nil => return Ok(Ast::Nil),
            Ast::Atom(ast) => return Ok(Ast::Atom(ast)),
        }
    }
}

fn eval_list(mut xs: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<EvalBehaviour, ReplError> {
    if xs.is_empty() {
        todo!("error: empty list")
    }

    if let Ast::Symbol(s) = &xs[0] {
        match s.as_str() {
            "def!" => Ok(EvalBehaviour::ReturnImmediately(eval_form_def(xs, env)?)),
            "let*" => do_form_let(xs, env),
            "letrec" => do_form_letrec(xs, env),
            "do" => do_form_do(xs, env),
            "if" => Ok(EvalBehaviour::LoopWithAst(do_form_if(xs, env)?)),
            "fun*" => Ok(EvalBehaviour::ReturnImmediately(eval_form_fun(xs, env)?)),
            "eval" => {
                let result = eval(xs.remove(1), env)?;
                Ok(EvalBehaviour::LoopWithAstAndEnv(result, get_root(env)))
            }
            _ => eval_func_call(xs, env),
        }
    } else {
        eval_func_call(xs, env)
    }
}

fn eval_form_def(mut args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    // todo arity check
    let definition = args.pop().unwrap();
    let name = get_symbol_name(args.pop().unwrap())?;

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
    let expr = args.pop().unwrap();
    let bindings = args.pop().unwrap();

    let n_env = bind_let(bindings, env, false)?;
    Ok(EvalBehaviour::LoopWithAstAndEnv(expr, n_env))
}

fn do_form_letrec(
    mut args: Vec<Ast>,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalBehaviour, ReplError> {
    let expr = args.pop().unwrap();
    let bindings = args.pop().unwrap();

    let n_env = bind_let(bindings, env, true)?;
    Ok(EvalBehaviour::LoopWithAstAndEnv(expr, n_env))
}

fn do_form_do(
    mut args: Vec<Ast>,
    env: &Rc<RefCell<Environment>>,
) -> Result<EvalBehaviour, ReplError> {
    let last_maybe = args.pop();

    for arg in args.into_iter().skip(1) {
        eval(arg, env)?;
    }

    if let Some(last) = last_maybe {
        Ok(EvalBehaviour::LoopWithAst(last))
    } else {
        Ok(EvalBehaviour::ReturnImmediately(Ast::Nil))
    }
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
    let body = args.pop().unwrap();
    let params = get_symbol_list(args.pop().unwrap())?;
    let fun = Ast::Function(Box::new(UserFunction {
        params,
        body,
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
            Ok(EvalBehaviour::LoopWithAstAndEnv(
                user_fun.body,
                Rc::new(RefCell::new(bind_fn(&user_fun.params, args, &user_fun.env))),
            ))
        }
        Ast::Builtin(name, cb) => Ok(EvalBehaviour::ReturnImmediately(cb(&name, args)?)),
        _ => todo!("error: attempted to call non-function"),
    }
}

fn eval_symbol(s: String, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let v = lookup(s, env)?;
    Ok(v)
}

// todo move into enum impl?
pub fn bind_fn(params: &[String], args: Vec<Ast>, env: &Rc<RefCell<Environment>>) -> Environment {
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

fn bind_let(
    ast: Ast,
    env: &Rc<RefCell<Environment>>,
    rec: bool,
) -> Result<Rc<RefCell<Environment>>, ReplError> {
    let n_env = Rc::new(RefCell::new(Environment {
        values: HashMap::new(),
        parent: Some(env.clone()),
    }));

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
            let v = eval(x, if rec { &n_env } else { env })?;
            n_env.borrow_mut().values.insert(symbol.clone(), v);
            get_sym = true;
        }
    }

    Ok(n_env)
}

fn get_symbol_name(ast: Ast) -> Result<String, ReplError> {
    match ast {
        Ast::Symbol(s) => Ok(s),
        _ => Err(ReplError::ParserError(ParserError::ExpectedSymbol)),
    }
}
