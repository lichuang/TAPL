use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    error::context,
    multi::many0,
    sequence::tuple,
};

use crate::{parser::IResult, typing::Type};

fn parse_boolean_type(input: &str) -> IResult<&str, Type> {
    context("parse_boolean_type", tag("Bool"))(input)
        .map(|(next_input, _res)| (next_input, Type::Boolean))
}

fn parse_number_type(input: &str) -> IResult<&str, Type> {
    context("parse_number_type", tag("Nat"))(input)
        .map(|(next_input, _res)| (next_input, Type::Number))
}

fn parse_atom_type(input: &str) -> IResult<&str, Type> {
    context(
        "parse_atom_type",
        alt((parse_boolean_type, parse_number_type)),
    )(input)
    .map(|(next_input, res)| (next_input, res))
}

fn parse_arrow_type(input: &str) -> IResult<&str, Type> {
    context(
        "parse_arrow_type",
        tuple((tag_no_case("->"), parse_atom_type)),
    )(input)
    .map(|(next_input, (_, res))| (next_input, res))
}

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    context(
        "parse_type",
        tuple((parse_atom_type, many0(parse_arrow_type))),
    )(input)
    .map(|(next_input, (typ, types))| {
        let mut lhs = typ;
        types.into_iter().map(|typ| lhs = typ);
        (next_input, lhs)
    })
}
