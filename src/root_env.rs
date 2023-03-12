use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

/* Helper functions */

fn get_int(ast: Ast, pos: u32, fn_name: &str) -> Result<i64, ParserError> {
    match ast {
        Ast::Integer(n) => Ok(n),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            pos,
            "Integer".to_owned(),
            ast.clone(),
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

pub fn lookup_ref(symbol: &str, env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    if let Some(v) = env.borrow().values.get(symbol) {
        Ok(v.clone())
    } else {
        match &env.borrow().parent {
            None => Err(ReplError::SymbolUndefined(symbol.to_owned())),
            Some(penv) => lookup_ref(symbol, penv),
        }
    }
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

fn prn(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = get_int(args.pop().unwrap(), 1, name)?;
    println!("{:?}", a);
    Ok(Ast::Nil)
}

fn op_eq(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
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

fn op_lt(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
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

fn list(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    Ok(Ast::List(args))
}

fn list_q(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    Ok(Ast::Boolean(matches!(a, Ast::List(_))))
}

fn empty_q(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    Ok(Ast::Boolean(matches!(a, Ast::List(xs) if xs.is_empty())))
}

fn count(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let a = args.pop().unwrap();
    Ok(Ast::Integer(if let Ast::List(xs) = a {
        xs.len() as i64
    } else {
        0
    }))
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

    Environment {
        values: root_env_table,
        parent: None,
    }
}
