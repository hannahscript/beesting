use crate::errors::ReplError;
use crate::root_env::Environment;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::iter::Peekable;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Symbol(String),
    Integer(i64),
    String(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

type PositionalToken = (usize, Token);

#[derive(Default)]
struct TokenizerState {
    tokens: Vec<PositionalToken>,
    buffer: String,
    quoting: bool,
}

impl TokenizerState {
    fn try_push(&mut self, c: char, index: usize) -> bool {
        if self.quoting {
            self.buffer.push(c);
            return false;
        }

        self.push_buffer(index, false);

        true
    }

    fn try_push_with(&mut self, c: char, index: usize, with: Token) {
        if self.try_push(c, index) {
            self.tokens.push((index, with))
        }
    }

    fn push_buffer(&mut self, index: usize, treat_as_str: bool) {
        if !self.buffer.is_empty() {
            self.tokens.push((
                index - self.buffer.len(),
                get_token(&self.buffer, treat_as_str),
            ));
            self.buffer.clear();
        }
    }
}

fn tokenize(text: &str) -> Vec<PositionalToken> {
    let mut state = TokenizerState::default();

    for (i, c) in text.char_indices() {
        match c {
            '\'' => {
                state.push_buffer(i, state.quoting);
                state.quoting = !state.quoting;
            }
            '(' => state.try_push_with(c, i, Token::LeftParen),
            ')' => state.try_push_with(c, i, Token::RightParen),
            _ => {
                if c.is_whitespace() {
                    state.try_push(c, i);
                } else {
                    state.buffer.push(c)
                }
            }
        }
    }

    state.push_buffer(text.len(), false);

    state.tokens
}

fn get_token(token: &str, is_str: bool) -> Token {
    if is_str {
        return Token::String(token.to_owned());
    }

    match token.parse::<i64>() {
        Ok(n) => Token::Integer(n),
        Err(_) => Token::Symbol(token.to_owned()),
    }
}

pub type EnvFunction = fn(&str, Vec<Ast>) -> Result<Ast, ReplError>;

#[derive(Clone)]
pub enum Ast {
    Symbol(String),
    Integer(i64),
    Boolean(bool),
    String(String),
    List(Vec<Ast>),
    Function(Box<UserFunction>),
    Builtin(String, EnvFunction),
    Nil,
}

#[derive(Clone)]
pub struct UserFunction {
    pub params: Vec<String>,
    pub body: Ast,
    pub env: Rc<RefCell<Environment>>,
}

impl Debug for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Integer(n) => write!(f, "{}", n),
            Ast::String(str) => write!(f, "{}", str),
            Ast::Function(_) => write!(f, "<function>"),
            Ast::Builtin(name, _) => write!(f, "<builtin:{}>", name),
            Ast::List(xs) => write!(f, "{:?}", xs),
            Ast::Symbol(s) => write!(f, "{}", s),
            Ast::Boolean(s) => write!(f, "{}", s),
            Ast::Nil => write!(f, "nil"),
        }
    }
}

pub enum ParserError {
    ExpectedGot(usize, Token, Token),
    ExpectedGotEof(Token),
    ExpectedAnyGotEof,
    TypeMismatch(String, u32, String, Ast),
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
        Token::String(str) => Ast::String(str),
    })
}

fn translate_symbol(symbol: &str) -> Ast {
    match symbol {
        "true" => Ast::Boolean(true),
        "false" => Ast::Boolean(false),
        "nil" => Ast::Nil,
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
