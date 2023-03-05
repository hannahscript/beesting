use crate::parser::{Ast, ParserError};
use std::collections::HashMap;

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

fn ensure_exact_arity(values: &[Ast], expected_arity: usize, fun: &str) -> Result<(), ParserError> {
    if values.len() != expected_arity {
        Err(ParserError::ArityMismatch(
            fun.to_owned(),
            expected_arity,
            values.len(),
        ))
    } else {
        Ok(())
    }
}

/* Standard lib */

fn add(name: &str, values: &[Ast]) -> Result<Ast, ParserError> {
    let mut total = 0;
    for (i, ast) in values.iter().enumerate() {
        let a = get_int(ast, i, name)?;
        total += a;
    }

    Ok(Ast::Integer(total))
}

fn sub(name: &str, values: &[Ast]) -> Result<Ast, ParserError> {
    ensure_exact_arity(values, 2, name)?;

    let a = get_int(values.get(0).unwrap(), 0, name)?;
    let b = get_int(values.get(1).unwrap(), 1, name)?;

    Ok(Ast::Integer(a - b))
}

fn mult(name: &str, values: &[Ast]) -> Result<Ast, ParserError> {
    let mut total = 0;
    for (i, ast) in values.iter().enumerate() {
        let a = get_int(ast, i, name)?;
        total *= a;
    }

    Ok(Ast::Integer(total))
}

fn div(name: &str, values: &[Ast]) -> Result<Ast, ParserError> {
    ensure_exact_arity(values, 2, name)?;

    let a = get_int(values.get(0).unwrap(), 0, name)?;
    let b = get_int(values.get(1).unwrap(), 1, name)?;

    Ok(Ast::Integer(a / b))
}

/* Public */

pub struct Environment<'a> {
    pub values: HashMap<&'a str, Ast>,
    pub parent: Option<Box<Environment<'a>>>,
}

pub fn create_root_env<'a>() -> Environment<'a> {
    let mut root_env_table = HashMap::new();
    root_env_table.insert("+", Ast::Function("add".to_owned(), add));
    root_env_table.insert("-", Ast::Function("sub".to_owned(), sub));
    root_env_table.insert("*", Ast::Function("mult".to_owned(), mult));
    root_env_table.insert("/", Ast::Function("div".to_owned(), div));

    Environment {
        values: root_env_table,
        parent: None,
    }
}
