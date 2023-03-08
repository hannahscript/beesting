use crate::errors::ReplError;
use crate::parser::{Ast, ParserError};
use std::collections::{HashMap, LinkedList};
use std::mem;

/* Helper functions */

fn get_int(ast: &Ast, index: usize, fn_name: &str) -> Result<i64, ParserError> {
    match ast {
        Ast::Integer(n) => Ok(*n),
        _ => Err(ParserError::TypeMismatch(
            fn_name.to_owned(),
            index + 1,
            "Integer".to_owned(),
            ast.clone(),
        )),
    }
}

pub fn lookup<'a>(symbol: &str, envs: &'a Environment) -> Result<&'a Ast, ReplError> {
    for env in envs {
        if let Some(v) = env.get(symbol) {
            return Ok(v);
        }
    }

    Err(ReplError::SymbolUndefined(symbol.to_owned()))
}

/* Standard lib */

fn add(env: &mut Environment) -> Result<Ast, ReplError> {
    // todo fix name arg
    let a = get_int(lookup("a", env)?, 0, "name")?;
    let b = get_int(lookup("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a + b))
}

fn sub(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = get_int(lookup("a", env)?, 0, "name")?;
    let b = get_int(lookup("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a - b))
}

fn mult(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = get_int(lookup("a", env)?, 0, "name")?;
    let b = get_int(lookup("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a * b))
}

fn div(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = get_int(lookup("a", env)?, 0, "name")?;
    let b = get_int(lookup("b", env)?, 1, "name")?;

    Ok(Ast::Integer(a / b))
}

fn prn(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = lookup("a", env)?;
    println!("{:?}", a);
    Ok(Ast::List(vec![]))
}

fn op_eq(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = lookup("a", env)?;
    let b = lookup("b", env)?;

    if mem::discriminant(a) != mem::discriminant(b) {
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

fn op_lt(env: &mut Environment) -> Result<Ast, ReplError> {
    let a = lookup("a", env)?;
    let b = lookup("b", env)?;

    if mem::discriminant(a) != mem::discriminant(b) {
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

pub type Environment = LinkedList<HashMap<String, Ast>>;

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
    LinkedList::from([root_env_table])
}
