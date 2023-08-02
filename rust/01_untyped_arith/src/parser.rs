use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::one_of,
    error::{context, VerboseError},
    multi::many_m_n,
    sequence::tuple,
    Err as NomErr,
};

pub type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Term {
    TmTrue,
    TmFalse,
    TmZero,
    TmSucc(Box<Term>),
    TmPred(Box<Term>),
    TmIsZero(Box<Term>),
    // condition term, then term, else term
    TmIf(Box<Term>, Box<Term>, Box<Term>),
}

impl Term {
    pub fn is_zero(&self) -> bool {
        self == &Term::TmZero
    }

    pub fn is_boolean(&self) -> bool {
        self == &Term::TmTrue || self == &Term::TmFalse
    }
}

impl From<&str> for Term {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_str() {
            "true" => Term::TmTrue,
            "false" => Term::TmFalse,
            "0" => Term::TmZero,
            _ => unimplemented!("no other value term supported"),
        }
    }
}

fn parse_succ(input: &str) -> IResult<&str, Term> {
    context(
        "succ",
        tuple((tag_no_case("succ"), tag("("), parse_term, tag(")"))),
    )(input)
    .map(|(next_input, (_, _, term, _))| (next_input, Term::TmSucc(Box::new(term))))
}

fn parse_pred(input: &str) -> IResult<&str, Term> {
    context(
        "pred",
        tuple((tag_no_case("pred"), tag("("), parse_term, tag(")"))),
    )(input)
    .map(|(next_input, (_, _, term, _))| (next_input, Term::TmPred(Box::new(term))))
}

fn parse_iszero(input: &str) -> IResult<&str, Term> {
    context(
        "iszero",
        tuple((tag_no_case("iszero"), tag("("), parse_term, tag(")"))),
    )(input)
    .map(|(next_input, (_, _, term, _))| (next_input, Term::TmIsZero(Box::new(term))))
}

fn parse_if(input: &str) -> IResult<&str, Term> {
    context(
        "if",
        tuple((
            tag_no_case("if "),
            parse_term,
            tag_no_case(" then "),
            parse_term,
            tag_no_case(" else "),
            parse_term,
        )),
    )(input)
    .map(|(next_input, (_, cond_term, _, then_term, _, else_term))| {
        (
            next_input,
            Term::TmIf(
                Box::new(cond_term),
                Box::new(then_term),
                Box::new(else_term),
            ),
        )
    })
}

fn parse_value(input: &str) -> IResult<&str, Term> {
    context(
        "parse_value",
        alt((tag_no_case("true"), tag_no_case("false"), tag_no_case("0"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn parse_numeric(input: &str) -> IResult<&str, Term> {
    fn n_to_m_digits<'a>(n: usize, m: usize) -> impl FnMut(&'a str) -> IResult<&str, String> {
        move |input| {
            many_m_n(n, m, one_of("0123456789"))(input)
                .map(|(next_input, result)| (next_input, result.into_iter().collect()))
        }
    }

    context("numeric", n_to_m_digits(1, 3))(input).and_then(|(next_input, result)| {
        match result.parse::<u8>() {
            Ok(n) => {
                let mut current_term = Term::TmSucc(Box::new(Term::TmZero));
                for _i in 1..n {
                    current_term = Term::TmSucc(Box::new(current_term));
                }

                Ok((next_input, current_term))
            }
            Err(_) => Err(NomErr::Error(VerboseError { errors: vec![] })),
        }
    })
}

fn parse_term(input: &str) -> IResult<&str, Term> {
    context(
        "term",
        alt((
            parse_value,
            parse_succ,
            parse_pred,
            parse_iszero,
            parse_if,
            parse_numeric,
        )),
    )(input)
    .map(|(next_input, res)| (next_input, res))
}

pub fn parse(input: &str) -> IResult<&str, Term> {
    context("parse", tuple((parse_term, tag(";"))))(input)
        .map(|(next_input, (term, _))| (next_input, term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term() {
        assert_eq!(parse_term("true"), Ok(("", Term::TmTrue)));
        assert_eq!(parse_term("FALSE"), Ok(("", Term::TmFalse)));
        assert_eq!(parse_term("0"), Ok(("", Term::TmZero)));
        assert_eq!(
            parse_term("succ(0)"),
            Ok(("", Term::TmSucc(Box::new(Term::TmZero))))
        );
        assert_eq!(
            parse_term("succ(2)"),
            Ok((
                "",
                Term::TmSucc(Box::new(Term::TmSucc(Box::new(Term::TmSucc(Box::new(
                    Term::TmZero
                ))))))
            ))
        );
        assert_eq!(
            parse_term("if false then true else false"),
            Ok((
                "",
                Term::TmIf(
                    Box::new(Term::TmFalse),
                    Box::new(Term::TmTrue),
                    Box::new(Term::TmFalse)
                )
            ))
        );
    }
}
