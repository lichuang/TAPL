use std::{collections::VecDeque, println};

use nom::{
    bytes::complete::tag,
    error::{context, VerboseError},
    sequence::tuple,
};

use crate::{
    ast_parser::{parse_term, ASTTerm},
    typing::Type,
};

pub type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

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
    // argument name, type and body
    TmAbs(String, Type, Box<Term>),
    TmApp(Box<Term>, Box<Term>),
    TmIf(Box<Term>, Box<Term>, Box<Term>),
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
                Term::TmAbs(arg.clone(), typ.clone(), Box::new(body_term))
            }
            ASTTerm::TmApp(left, right) => {
                let left = self.from_ast_term(&left.as_ref())?;
                let right = self.from_ast_term(&right.as_ref())?;
                Term::TmApp(Box::new(left), Box::new(right))
            }
            ASTTerm::TmIf(if_term, then_term, else_them) => {
                let if_term = self.from_ast_term(&if_term.as_ref())?;
                let then_term = self.from_ast_term(&then_term.as_ref())?;
                let else_them = self.from_ast_term(&else_them.as_ref())?;
                Term::TmIf(Box::new(if_term), Box::new(then_term), Box::new(else_them))
            }
        };

        Ok(term)
    }
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
            let input = "(lambda x:Bool.x);";
            let term = parser.parse(input);
            println!("term: {:?}", term);
            //assert_eq!(term, Ok(Term::TmZero));
        }
        {
            let mut parser = Parser::new();
            let input = "lambda x:bool.if x then false else true;";
            let term = parser.parse(input);
            println!("term: {:?}", term);
            //assert_eq!(term, Ok(Term::TmZero));
        }
    }
}
