use std::fmt::Formatter;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::{io, mem};

#[derive(Debug)]
enum AST {
    Symbol(String),
    Integer(i64),
    List(Vec<AST>),
}

enum ParserError {
    ExpectedGot(usize, Token, Token),
    ExpectedGotEof(Token),
    ExpectedAnyGotEof,
    //NotANumber(usize, char),
}

#[derive(Debug)]
enum ReplError {
    ParserError(ParserError),
    IoError(io::Error),
}

impl std::fmt::Debug for ParserError {
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

type PositionalToken = (usize, Token);

fn peek(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<&Token, ParserError> {
    Ok(&it.peek().ok_or(ParserError::ExpectedAnyGotEof)?.1)
}

fn next(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<Token, ParserError> {
    match it.next() {
        None => Err(ParserError::ExpectedAnyGotEof),
        Some((_i, t)) => Ok(t),
    }
}

fn parse_list(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<AST, ParserError> {
    expect(it, Token::LeftParen)?;

    let mut items = vec![];
    while *peek(it)? != Token::RightParen {
        items.push(parse_any(it)?);
    }

    expect(it, Token::RightParen)?;

    Ok(AST::List(items))
}

fn parse_atom(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<AST, ParserError> {
    let atom = next(it)?;

    Ok(match atom {
        Token::LeftParen => panic!("wtf"),
        Token::RightParen => panic!("wtf"),
        Token::Symbol(s) => AST::Symbol(s),
        Token::Integer(n) => AST::Integer(n),
    })
}

fn parse_any(it: &mut Peekable<IntoIter<PositionalToken>>) -> Result<AST, ParserError> {
    let next = peek(it)?;

    if *next == Token::LeftParen {
        parse_list(it)
    } else {
        parse_atom(it)
    }
}

#[derive(Debug)]
enum Token {
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

fn get_token(token: &String) -> Token {
    match token.parse::<i64>() {
        Ok(n) => Token::Integer(n),
        Err(_) => Token::Symbol(token.clone()),
    }
}

fn read() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn eval(text: String) -> Result<AST, ParserError> {
    let tokens = tokenize(text.trim());
    println!("Tokens: {:?}", tokens);
    let mut it = tokens.into_iter().peekable();
    parse_any(&mut it)
}

fn rep() -> Result<AST, ReplError> {
    let input = match read() {
        Ok(inp) => Ok(inp),
        Err(err) => Err(ReplError::IoError(err)),
    }?;

    match eval(input) {
        Ok(ast) => Ok(ast),
        Err(err) => Err(ReplError::ParserError(err)),
    }
}

fn main() {
    loop {
        print!("user> ");
        let output_result = rep();
        match output_result {
            Ok(output) => println!("{:?}", output),
            Err(err) => eprintln!("Error occurred: {:?}", err),
        }
    }
}
