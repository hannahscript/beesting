use crate::errors::ReplError;
use crate::root_env::Environment;
use std::fmt::{Debug, Formatter};
use std::iter::Peekable;
use std::mem;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Symbol(String),
    Integer(i64),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

type PositionalToken = (usize, Token);

fn tokenize(text: &str) -> Vec<PositionalToken> {
    let mut tokens = vec![];
    let mut buffer = String::new();

    for (i, c) in text.char_indices() {
        match c {
            ' ' => {
                if !buffer.is_empty() {
                    tokens.push((i - buffer.len(), get_token(&buffer)));
                    buffer.clear();
                }
            }
            '(' => {
                if !buffer.is_empty() {
                    tokens.push((i - buffer.len(), get_token(&buffer)));
                    buffer.clear();
                }
                tokens.push((i, Token::LeftParen))
            }
            ')' => {
                if !buffer.is_empty() {
                    tokens.push((i - buffer.len(), get_token(&buffer)));
                    buffer.clear();
                }
                tokens.push((i, Token::RightParen))
            }
            _ => buffer.push(c),
        }
    }

    if !buffer.is_empty() {
        tokens.push((text.len() - buffer.len(), get_token(&buffer)));
    }

    tokens
}

fn get_token(token: &str) -> Token {
    match token.parse::<i64>() {
        Ok(n) => Token::Integer(n),
        Err(_) => Token::Symbol(token.to_owned()),
    }
}

pub type Function = fn(&str, &[Ast]) -> Result<Ast, ParserError>;
pub type EnvFunction = fn(&mut Environment) -> Result<Ast, ReplError>;

#[derive(Clone)]
pub enum Ast {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
    List(Vec<Ast>),
    Function(Box<UserFunction>),
    Builtin(Vec<String>, EnvFunction),
}

#[derive(Clone)]
pub struct UserFunction {
    pub params: Vec<String>,
    pub body: Ast,
}

impl Debug for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Integer(n) => write!(f, "{}", n),
            Ast::Function(_) => write!(f, "<function>"),
            Ast::Builtin(_, _) => write!(f, "<builtin>"),
            Ast::List(xs) => write!(f, "{:?}", xs),
            Ast::Symbol(s) => write!(f, "{}", s),
            Ast::Boolean(s) => write!(f, "{}", s),
        }
    }
}

pub enum ParserError {
    ExpectedGot(usize, Token, Token),
    ExpectedGotEof(Token),
    ExpectedAnyGotEof,
    TypeMismatch(String, usize, String, Ast),
    ArityMismatch(String, usize, usize),
    ExpectedSymbol,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::ExpectedGot(pos, expected, actual) => write!(
                f,
                "Error on position {}: Expected '{:?}', but got '{:?}'",
                pos, expected, actual
            ),
            ParserError::ExpectedGotEof(expected) => {
                write!(f, "Error: Expected '{:?}', but got EOF", expected)
            }
            ParserError::ExpectedAnyGotEof => write!(f, "Error: Expected any input but got EOF"),

            ParserError::TypeMismatch(fn_name, index, expected, got) => write!(
                f,
                "Type mismatch: Expected {} at argument position {} of {} but got {:?}",
                expected, index, fn_name, got
            ),
            ParserError::ArityMismatch(fun, expected, got) => write!(
                f,
                "Arity mismatch: Expected {} arguments for function <function:{}> but got {}",
                fun, expected, got
            ),
            ParserError::ExpectedSymbol => write!(f, "Expected symbol"),
        }
    }
}

fn expect(
    it: &mut Peekable<IntoIter<PositionalToken>>,
    expected: Token,
) -> Result<(), ParserError> {
    match it.next() {
        None => Err(ParserError::ExpectedGotEof(expected)),
        Some((i, t)) => {
            if t == expected {
                Ok(())
            } else {
                Err(ParserError::ExpectedGot(i, expected, t))
            }
        }
    }
}

fn peek(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<&Token, ParserError> {
    Ok(&it.peek().ok_or(ParserError::ExpectedAnyGotEof)?.1)
}

fn next(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<Token, ParserError> {
    match it.next() {
        None => Err(ParserError::ExpectedAnyGotEof),
        Some((_i, t)) => Ok(t),
    }
}

fn parse_list(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<Ast, ParserError> {
    expect(it, Token::LeftParen)?;

    let mut items = vec![];
    while *peek(it)? != Token::RightParen {
        items.push(parse_any(it)?);
    }

    expect(it, Token::RightParen)?;

    Ok(Ast::List(items))
}

fn parse_atom(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<Ast, ParserError> {
    let atom = next(it)?;

    Ok(match atom {
        Token::LeftParen => panic!("wtf"),
        Token::RightParen => panic!("wtf"),
        Token::Symbol(s) => translate_symbol(&s),
        Token::Integer(n) => Ast::Integer(n),
    })
}

fn translate_symbol(symbol: &str) -> Ast {
    match symbol {
        "true" => Ast::Boolean(true),
        "false" => Ast::Boolean(false),
        other => Ast::Symbol(other.to_owned()),
    }
}

fn parse_any(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<Ast, ParserError> {
    let next = peek(it)?;

    if *next == Token::LeftParen {
        parse_list(it)
    } else {
        parse_atom(it)
    }
}

impl FromStr for Ast {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenize(s.trim());
        let mut it = tokens.into_iter().peekable();
        parse_any(&mut it)
    }
}
