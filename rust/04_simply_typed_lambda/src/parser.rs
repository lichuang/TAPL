use std::{collections::VecDeque, println};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha0, alpha1, anychar, char as NomChar, multispace0, one_of},
    error::{context, VerboseError, VerboseErrorKind},
    multi::many1,
    sequence::tuple,
};

use misc::ALPHABET;
use nom::Err as NomErr;

use crate::{type_parser::parse_type, typing::Type};

pub type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ASTTerm {
    TmTrue,
    TmFalse,
    TmZero,
    TmSucc(Box<ASTTerm>),
    TmVar(String),
    // argument ident, argument type and body
    TmAbs(String, Type, Box<ASTTerm>),
    TmApp(Box<ASTTerm>, Box<ASTTerm>),
}

impl From<&str> for ASTTerm {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_str() {
            "true" => ASTTerm::TmTrue,
            "false" => ASTTerm::TmFalse,
            "0" => ASTTerm::TmZero,
            _ => unimplemented!("no other value term supported"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DeBruijnIndexer {
    inner: VecDeque<String>,
}

impl DeBruijnIndexer {
    pub fn push(&mut self, hint: String) -> usize {
        if self.inner.contains(&hint) {
            self.push(hint)
        } else {
            let idx = self.inner.len();
            self.inner.push_front(hint);
            idx
        }
    }

    pub fn pop(&mut self) {
        self.inner.pop_front();
    }

    pub fn lookup(&self, key: &str) -> Option<usize> {
        for (idx, s) in self.inner.iter().enumerate() {
            if key == s {
                return Some(idx);
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Term {
    TmTrue,
    TmFalse,
    TmZero,
    TmSucc(Box<Term>),
    // var DeBrujin index
    TmVar(usize),
    // argument type and body
    TmAbs(Type, Box<Term>),
    TmApp(Box<Term>, Box<Term>),
}

pub struct Parser {
    context: DeBruijnIndexer,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ParseError {
    VerboseError(String),
    UnboundVariable(String),
}

impl From<nom::Err<VerboseError<&str>>> for ParseError {
    fn from(i: nom::Err<VerboseError<&str>>) -> Self {
        ParseError::VerboseError(i.to_string())
    }
}

pub type ParseResult = Result<Term, ParseError>;

impl Parser {
    pub fn new() -> Self {
        Self {
            context: DeBruijnIndexer::default(),
        }
    }

    pub fn parse(self: &mut Parser, input: &str) -> ParseResult {
        //println!("parse");
        let (output, term) = context("parse", tuple((parse_term, tag(";"))))(input)
            .map(|(next_input, (term, _))| (next_input, term))?;

        assert!(output.is_empty());

        self.from_ast_term(&term)
    }

    fn from_ast_term(self: &mut Parser, ast_term: &ASTTerm) -> ParseResult {
        let term = match ast_term {
            ASTTerm::TmTrue => Term::TmTrue,
            ASTTerm::TmFalse => Term::TmFalse,
            ASTTerm::TmZero => Term::TmZero,
            ASTTerm::TmSucc(number) => {
                let term = self.from_ast_term(number.as_ref())?;
                Term::TmSucc(Box::new(term))
            }
            ASTTerm::TmVar(id) => match self.context.lookup(&id) {
                Some(index) => Term::TmVar(index),
                None => {
                    return Err(ParseError::UnboundVariable(id.to_string()));
                }
            },
            ASTTerm::TmAbs(arg, typ, body) => {
                // Bind variable into a new context before parsing the body
                self.context.push(arg.to_string());
                let body_term = self.from_ast_term(body.as_ref())?;
                // Return to previous context
                self.context.pop();
                Term::TmAbs(typ.clone(), Box::new(body_term))
            }
            ASTTerm::TmApp(left, right) => {
                let left = self.from_ast_term(&left.as_ref())?;
                let right = self.from_ast_term(&right.as_ref())?;
                Term::TmApp(Box::new(left), Box::new(right))
            }
        };

        Ok(term)
    }
}

fn parse_value(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_value {:?}", input);
    context(
        "parse_value",
        alt((tag("true"), tag("false"), tag_no_case("0"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn parse_succ(input: &str) -> IResult<&str, ASTTerm> {
    context(
        "parse_succ",
        tuple((tag_no_case("succ"), tag("("), parse_term, tag(")"))),
    )(input)
    .map(|(next_input, (_, _, term, _))| (next_input, ASTTerm::TmSucc(Box::new(term))))
}

fn parse_ident(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_ident {:?}", input);
    context("parse_ident", tuple((multispace0, one_of(ALPHABET))))(input)
        .map(|(next_input, (_, res))| (next_input, ASTTerm::TmVar(res.to_string())))
}

fn parse_atom(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_atom {:?}", input);
    context(
        "parse_atom",
        alt((parse_value, parse_succ, parse_ident, parse_paren_term)),
    )(input)
    .map(|(next_input, res)| (next_input, res))
}

fn parse_paren_term(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_paren_term {:?}", input);
    context("parse_paren_term", tuple((tag("("), parse_term, tag(")"))))(input)
        .map(|(next_input, (_, term, _))| (next_input, term))
}

fn parse_abstraction(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_abstraction: {:?}", input);
    context(
        "parse_abstraction",
        tuple((
            tag("lambda "),
            one_of(ALPHABET),
            tag_no_case(":"),
            parse_type,
            tag("."),
            parse_term,
        )),
    )(input)
    .map(|(next_input, (_, param, _, typ, _, body))| {
        println!("param: {:?}, typ: {:?}", param, typ);
        (
            next_input,
            ASTTerm::TmAbs(param.to_string(), typ, Box::new(body)),
        )
    })
}

fn parse_application(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_application {:?}", input);
    context("parse_application", many1(parse_atom))(input).map(|(next_input, vars)| {
        //println!("vars: {:?}", vars);
        let mut lhs = Box::new(vars[0].clone());
        let mut i = 1;
        while i < vars.len() {
            let rhs = Box::new(vars[i].clone());
            lhs = Box::new(ASTTerm::TmApp(lhs, rhs));
            i += 1;
        }
        (next_input, lhs.as_ref().clone())
    })
}

fn parse_term(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_term: {:?}", input);
    context("term", alt((parse_abstraction, parse_application)))(input)
        .map(|(next_input, res)| (next_input, res))
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;

    #[test]
    fn test_term() {
        {
            let mut parser = Parser::new();
            assert_eq!(parser.parse("true;"), Ok(Term::TmTrue));
        }
        {
            let mut parser = Parser::new();
            assert_eq!(parser.parse("0;"), Ok(Term::TmZero));
        }
        {
            let mut parser = Parser::new();
            let input = "(lambda x:Bool.y);";
            let term = parser.parse(input);
            println!("term: {:?}", term);
            assert!(false);
            //assert_eq!(term, Ok(Term::TmZero));
        }
    }
}
