use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha0, alpha1, multispace0, one_of},
    error::{context, VerboseError},
    multi::many1,
    sequence::tuple,
};

use misc::ALPHABET;
pub type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Term {
    TmVar(String),
    // argument and body
    TmAbs(String, Box<Term>),
    TmApp(Box<Term>, Box<Term>),
}

fn parse_paren_term(input: &str) -> IResult<&str, Term> {
    //println!("parse_paren_term {:?}", input);
    context("parse_paren_term", tuple((tag("("), parse_term, tag(")"))))(input)
        .map(|(next_input, (_, term, _))| (next_input, term))
}

fn parse_variable(input: &str) -> IResult<&str, Term> {
    println!("parse_variable {:?}", input);
    context("parse_ident", tuple((multispace0, one_of(ALPHABET))))(input)
        .map(|(next_input, (_, res))| (next_input, Term::TmVar(res.to_string())))
}

fn parse_atom(input: &str) -> IResult<&str, Term> {
    //println!("parse_atom {:?}", input);
    context("parse_atom", alt((parse_variable, parse_paren_term)))(input)
        .map(|(next_input, res)| (next_input, res))
}

fn parse_abstraction(input: &str) -> IResult<&str, Term> {
    println!("parse_abstraction: {:?}", input);
    context(
        "parse_abstraction",
        tuple((
            tag_no_case("lambda "),
            one_of(ALPHABET),
            tag("."),
            parse_term,
        )),
    )(input)
    .map(|(next_input, (_, param, _, body))| {
        /*
        println!(
            "param: {:?}, body: {:?}, next_input: {:?}",
            param, body, next_input
        );
        */
        (next_input, Term::TmAbs(param.to_string(), Box::new(body)))
    })
}

fn parse_application(input: &str) -> IResult<&str, Term> {
    //println!("parse_application {:?}", input);
    context("parse_application", many1(parse_atom))(input).map(|(next_input, vars)| {
        //println!("vars: {:?}", vars);
        let mut lhs = Box::new(vars[0].clone());
        for i in 1..vars.len() {
            let rhs = Box::new(vars[i].clone());
            lhs = Box::new(Term::TmApp(lhs, rhs));
        }
        (next_input, lhs.as_ref().clone())
    })
}

fn parse_term(input: &str) -> IResult<&str, Term> {
    //println!("parse_term: {:?}", input);
    context("term", alt((parse_abstraction, parse_application)))(input)
        .map(|(next_input, res)| (next_input, res))
}

pub fn parse(input: &str) -> IResult<&str, Term> {
    //println!("parse");
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
            parse("(lambda x.(lambda y.y)) z;"),
            Ok((
                "",
                Term::TmApp(
                    Box::new(Term::TmAbs(
                        "x".to_string(),
                        Box::new(Term::TmAbs(
                            "y".to_string(),
                            Box::new(Term::TmVar("y".to_string()))
                        ))
                    )),
                    Box::new(Term::TmVar("z".to_string()))
                )
            ))
        );
    }
}
