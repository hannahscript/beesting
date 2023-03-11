use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

/* Helper functions */

fn get_int(ast: Ast, index: usize, fn_name: &str) -> Result<i64, ParserError> {
    match ast {
        Ast::Integer(n) => Ok(n),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            index + 1,
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

fn add(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    // todo fix name arg
    let a = get_int(lookup_ref("a", env)?, 0, "name")?;
    let b = get_int(lookup_ref("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a + b))
}

fn sub(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = get_int(lookup_ref("a", env)?, 0, "name")?;
    let b = get_int(lookup_ref("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a - b))
}

fn mult(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = get_int(lookup_ref("a", env)?, 0, "name")?;
    let b = get_int(lookup_ref("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a * b))
}

fn div(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = get_int(lookup_ref("a", env)?, 0, "name")?;
    let b = get_int(lookup_ref("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a / b))
}

fn prn(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = lookup_ref("a", env)?;
    println!("{:?}", a);
    Ok(Ast::List(vec![]))
}

fn op_eq(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = lookup_ref("a", env)?;
    let b = lookup_ref("b", env)?;

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

fn op_lt(env: &Rc<RefCell<Environment>>) -> Result<Ast, ReplError> {
    let a = lookup_ref("a", env)?;
    let b = lookup_ref("b", env)?;

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

/* Public */

#[derive(Clone)]
pub struct Environment {
    pub values: HashMap<String, Ast>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

pub fn create_root_env() -> Environment {
    let mut root_env_table = HashMap::new();
    // root_env_table.insert("+".to_owned(), Ast::Function("add".to_owned(), add));
    root_env_table.insert(
        "+".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], add),
    );
    root_env_table.insert(
        "-".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], sub),
    );
    root_env_table.insert(
        "*".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], mult),
    );
    root_env_table.insert(
        "/".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], div),
    );
    root_env_table.insert("prn".to_owned(), Ast::Builtin(vec!["a".to_owned()], prn));
    root_env_table.insert(
        "=".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], op_eq),
    );
    root_env_table.insert(
        "<".to_owned(),
        Ast::Builtin(vec!["a".to_owned(), "b".to_owned()], op_lt),
    );

    Environment {
        values: root_env_table,
        parent: None,
    }
}
