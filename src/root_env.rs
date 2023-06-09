use crate::errors::ReplError;
use crate::eval::{bind_fn, eval};
use crate::parser::{Ast, ParserError, UserFunction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fs, mem};

/* Helper functions */

fn get_int(ast: Ast, pos: u32, fn_name: &str) -> Result<i64, ParserError> {
    match ast {
        Ast::Integer(n) => Ok(n),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            pos,
            "Integer".to_owned(),
            ast,
        )),
    }
}

fn get_str(ast: Ast, pos: u32, fn_name: &str) -> Result<String, ParserError> {
    match ast {
        Ast::String(str) => Ok(str),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            pos,
            "String".to_owned(),
            ast,
        )),
    }
}

fn get_atom(ast: Ast, pos: u32, fn_name: &str) -> Result<Rc<RefCell<Ast>>, ParserError> {
    match ast {
        Ast::Atom(ast) => Ok(ast),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            pos,
            "Atom".to_owned(),
            ast,
        )),
    }
}

fn get_fun(ast: Ast, pos: u32, fn_name: &str) -> Result<Box<UserFunction>, ParserError> {
    match ast {
        Ast::Function(ast) => Ok(ast),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            pos,
            "Function".to_owned(),
            ast,
        )),
    }
}

pub fn lookup(symbol: String, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    if let Some(v) = env.borrow().values.get(&symbol) {
        Ok(v.clone())
    } else {
        match &env.borrow().parent {
            None => Err(ReplError::SymbolUndefined(symbol.to_owned())),
            Some(penv) => lookup(symbol, penv),
        }
    }
}

pub fn get_root(env: &Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
    let mut current_env = Rc::clone(env);

    loop {
        current_env = {
            let parent_maybe = &current_env.borrow().parent;
            if parent_maybe.is_none() {
                break;
            }

            Rc::clone(parent_maybe.as_ref().unwrap())
        }
    }

    Rc::clone(&current_env)
}

/* Standard lib */

fn add(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = get_int(args.pop().unwrap(), 2, name)?;
    let a = get_int(args.pop().unwrap(), 1, name)?;

    Ok(Ast::Integer(a + b))
}

fn sub(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = get_int(args.pop().unwrap(), 2, name)?;
    let a = get_int(args.pop().unwrap(), 1, name)?;

    Ok(Ast::Integer(a - b))
}

fn mult(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = get_int(args.pop().unwrap(), 2, name)?;
    let a = get_int(args.pop().unwrap(), 1, name)?;

    Ok(Ast::Integer(a * b))
}

fn div(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = get_int(args.pop().unwrap(), 2, name)?;
    let a = get_int(args.pop().unwrap(), 1, name)?;

    Ok(Ast::Integer(a / b))
}

fn prn(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    println!("{:?}", a);
    Ok(Ast::Nil)
}

fn op_eq(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = args.pop().unwrap();
    let a = args.pop().unwrap();

    if mem::discriminant(&a) != mem::discriminant(&b) {
        return Ok(Ast::Boolean(false));
    }

    match a {
        Ast::Integer(a_n) => {
            if let Ast::Integer(b_n) = b {
                Ok(Ast::Boolean(a_n == b_n))
            } else {
                Ok(Ast::Boolean(false))
            }
        }
        Ast::Boolean(a_b) => {
            if let Ast::Boolean(b_b) = b {
                Ok(Ast::Boolean(a_b == b_b))
            } else {
                Ok(Ast::Boolean(false))
            }
        }
        _ => Ok(Ast::Boolean(false)),
    }
}

fn op_lt(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = args.pop().unwrap();
    let a = args.pop().unwrap();

    if mem::discriminant(&a) != mem::discriminant(&b) {
        return Ok(Ast::Boolean(false));
    }

    match a {
        Ast::Integer(a_n) => {
            if let Ast::Integer(b_n) = b {
                Ok(Ast::Boolean(a_n < b_n))
            } else {
                Ok(Ast::Boolean(false))
            }
        }
        _ => Ok(Ast::Boolean(false)),
    }
}

fn list(_name: &str, args: Vec<Ast>) -> Result<Ast, ReplError> {
    Ok(Ast::List(args))
}

fn list_q(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    Ok(Ast::Boolean(matches!(a, Ast::List(_))))
}

fn empty_q(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    Ok(Ast::Boolean(matches!(a, Ast::List(xs) if xs.is_empty())))
}

fn count(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();

    Ok(Ast::Integer(if let Ast::List(xs) = a {
        xs.len() as i64
    } else {
        0
    }))
}

fn concat_str(name: &str, args: Vec<Ast>) -> Result<Ast, ReplError> {
    let mut str = String::new();
    for (i, arg) in args.into_iter().enumerate() {
        str += &get_str(arg, (i as u32) + 1, name)?;
    }

    Ok(Ast::String(str))
}

fn slurp(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let file_name = get_str(args.pop().unwrap(), 1, name)?;

    let content = fs::read_to_string(file_name)?;
    Ok(Ast::String(content))
}

fn read_str(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = get_str(args.pop().unwrap(), 1, name)?;

    Ok(a.parse()?)
}

/* Atom */
fn atom(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();

    Ok(Ast::Atom(Rc::new(RefCell::new(a))))
}

fn atom_q(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();

    Ok(Ast::Boolean(matches!(a, Ast::Atom(_))))
}

fn deref(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();

    if let Ast::Atom(ast) = a {
        Ok(ast.borrow().to_owned())
    } else {
        todo!("Error: not an atom")
    }
}

fn reset_m(_name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let val = args.pop().unwrap();
    let atom = args.pop().unwrap();

    if let Ast::Atom(ast) = atom {
        *ast.borrow_mut() = val.clone();
        Ok(val)
    } else {
        todo!("Error: not an atom")
    }
}

fn swap_m(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let fun = get_fun(args.pop().unwrap(), 2, name)?;
    let atom = get_atom(args.pop().unwrap(), 1, name)?;

    let atom_content = atom.borrow_mut().clone();
    let env = bind_fn(&fun.params, vec![atom_content], &fun.env);
    let new_val = eval(fun.body, &Rc::new(RefCell::new(env)))?;
    *atom.borrow_mut() = new_val;
    Ok(Ast::Atom(atom))
}

/* Public */

#[derive(Clone)]
pub struct Environment {
    pub values: HashMap<String, Ast>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

pub fn create_root_env() -> Environment {
    let mut root_env_table = HashMap::new();
    root_env_table.insert("+".to_owned(), Ast::Builtin("+".to_owned(), add));
    root_env_table.insert("-".to_owned(), Ast::Builtin("-".to_owned(), sub));
    root_env_table.insert("*".to_owned(), Ast::Builtin("*".to_owned(), mult));
    root_env_table.insert("/".to_owned(), Ast::Builtin("/".to_owned(), div));
    root_env_table.insert("prn".to_owned(), Ast::Builtin("prn".to_owned(), prn));
    root_env_table.insert("=".to_owned(), Ast::Builtin("=".to_owned(), op_eq));
    root_env_table.insert("<".to_owned(), Ast::Builtin("<".to_owned(), op_lt));
    root_env_table.insert("list".to_owned(), Ast::Builtin("list".to_owned(), list));
    root_env_table.insert("list?".to_owned(), Ast::Builtin("list?".to_owned(), list_q));
    root_env_table.insert(
        "empty?".to_owned(),
        Ast::Builtin("empty?".to_owned(), empty_q),
    );
    root_env_table.insert("count".to_owned(), Ast::Builtin("count".to_owned(), count));
    root_env_table.insert("str".to_owned(), Ast::Builtin("str".to_owned(), concat_str));
    root_env_table.insert("slurp".to_owned(), Ast::Builtin("slurp".to_owned(), slurp));
    root_env_table.insert(
        "read-str".to_owned(),
        Ast::Builtin("read-str".to_owned(), read_str),
    );

    root_env_table.insert("atom".to_owned(), Ast::Builtin("atom".to_owned(), atom));
    root_env_table.insert("atom?".to_owned(), Ast::Builtin("atom?".to_owned(), atom_q));
    root_env_table.insert("deref".to_owned(), Ast::Builtin("deref".to_owned(), deref));
    root_env_table.insert(
        "reset!".to_owned(),
        Ast::Builtin("reset!".to_owned(), reset_m),
    );
    root_env_table.insert("swap!".to_owned(), Ast::Builtin("swap!".to_owned(), swap_m));

    Environment {
        values: root_env_table,
        parent: None,
    }
}
