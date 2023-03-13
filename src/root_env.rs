use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
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

fn concat_str(name: &str, mut args: Vec<Ast>) -> Result<Ast, ReplError> {
    let b = get_str(args.pop().unwrap(), 2, name)?;
    let mut a = get_str(args.pop().unwrap(), 1, name)?;

    a.push_str(&b);
    Ok(Ast::String(a))
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
    root_env_table.insert(".".to_owned(), Ast::Builtin(".".to_owned(), concat_str));
    root_env_table.insert("slurp".to_owned(), Ast::Builtin("slurp".to_owned(), slurp));
    root_env_table.insert(
        "read-str".to_owned(),
        Ast::Builtin("read-str".to_owned(), read_str),
    );

    Environment {
        values: root_env_table,
        parent: None,
    }
}
