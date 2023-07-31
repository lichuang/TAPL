use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    error::{context, VerboseError},
    multi::many_m_n,
    sequence::tuple,
};

use nom::character::complete::anychar;

pub type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Term {
    TmVar(String),
    // argument and body
    TmAbs(String, Box<Term>),
    TmApp(Box<Term>, Box<Term>),
}

fn parse_variable(input: &str) -> IResult<&str, Term> {
    context("parse_variable", anychar)(input)
        .map(|(next_input, res)| (next_input, Term::TmVar(res.to_string())))
}

fn parse_abstraction(input: &str) -> IResult<&str, Term> {
    println!("parse_abstraction: {:?}", input);
    context(
        "parse_abstraction",
        tuple((
            tag("("),
            tag_no_case("lambda "),
            anychar,
            tag("."),
            parse_variable,
            tag(")"),
        )),
    )(input)
    .map(|(next_input, (_, _, param, _, body, _))| {
        println!("param: {:?}", param);
        (next_input, Term::TmAbs(param.to_string(), Box::new(body)))
    })
}

fn parse_application(input: &str) -> IResult<&str, Term> {
    context("parse_application", many_m_n(2, 1024, parse_variable))(input).map(
        |(next_input, vars)| {
            println!("vars: {:?}", vars);

            let mut lhs = Box::new(vars[0].clone());
            for i in 1..vars.len() {
                let rhs = Box::new(vars[i].clone());
                lhs = Box::new(Term::TmApp(lhs, rhs));
            }
            (next_input, lhs.as_ref().clone())
        },
    )
}

fn parse_term(input: &str) -> IResult<&str, Term> {
    println!("parse_term: {:?}", input);
    context("term", alt((parse_abstraction, parse_application)))(input)
        .map(|(next_input, res)| (next_input, res))
}

pub fn parse(input: &str) -> IResult<&str, Term> {
    println!("parse");
    context("parse", tuple((parse_term, tag(";"))))(input)
        .map(|(next_input, (term, _))| (next_input, term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term() {
        assert_eq!(
            parse("(lambda x.y);"),
            Ok((
                "",
                Term::TmAbs("x".to_string(), Box::new(Term::TmVar("y".to_string())))
            ))
        );

        assert_eq!(
            parse("(lambda x.(lambda y.y) z);"),
            Ok((
                "",
                Term::TmAbs("x".to_string(), Box::new(Term::TmVar("y".to_string())))
            ))
        );
    }
}
