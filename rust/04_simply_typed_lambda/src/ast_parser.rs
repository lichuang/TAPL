use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{multispace0, one_of},
    error::{context, VerboseError},
    multi::many1,
    sequence::tuple,
};

use misc::ALPHABET;

use crate::{parser::IResult, type_parser::parse_type, typing::Type};

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
    // condition term, then term, else term
    TmIf(Box<ASTTerm>, Box<ASTTerm>, Box<ASTTerm>),
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

fn parse_value(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_value {:?}", input);
    context(
        "parse_value",
        alt((tag("true"), tag("false"), tag_no_case("0"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn parse_succ(input: &str) -> IResult<&str, ASTTerm> {
    context(
        "parse_succ",
        tuple((tag("succ"), tag("("), parse_term, tag(")"))),
    )(input)
    .map(|(next_input, (_, _, term, _))| (next_input, ASTTerm::TmSucc(Box::new(term))))
}

fn parse_ident(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_ident {:?}", input);
    context("parse_ident", tuple((multispace0, one_of(ALPHABET))))(input)
        .map(|(next_input, (_, res))| (next_input, ASTTerm::TmVar(res.to_string())))
}

fn parse_if(input: &str) -> IResult<&str, ASTTerm> {
    context(
        "parse_if",
        tuple((
            tag("if "),
            parse_term,
            tag(" then "),
            parse_term,
            tag(" else "),
            parse_term,
        )),
    )(input)
    .map(|(next_input, (_, condition, _, then_term, _, else_term))| {
        (
            next_input,
            ASTTerm::TmIf(
                Box::new(condition),
                Box::new(then_term),
                Box::new(else_term),
            ),
        )
    })
}

fn parse_atom(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_atom {:?}", input);
    context(
        "parse_atom",
        alt((
            parse_value,
            parse_succ,
            parse_ident,
            parse_if,
            parse_parent_term,
        )),
    )(input)
    .map(|(next_input, res)| (next_input, res))
}

fn parse_parent_term(input: &str) -> IResult<&str, ASTTerm> {
    //println!("parse_paren_term {:?}", input);
    context("parse_parent_term", tuple((tag("("), parse_term, tag(")"))))(input)
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

pub fn parse_term(input: &str) -> IResult<&str, ASTTerm> {
    println!("parse_term: {:?}", input);
    context("term", alt((parse_abstraction, parse_application)))(input)
        .map(|(next_input, res)| (next_input, res))
}
