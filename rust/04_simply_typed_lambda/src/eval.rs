use crate::{
    context::Context,
    parser::Term,
    typing::{Type, TypeError},
};

use nom::error::VerboseError;

pub enum EvalError {
    VerboseError(String),
    TypeError(String),
    NoRuleApplies,
}

impl From<nom::Err<VerboseError<&str>>> for EvalError {
    fn from(i: nom::Err<VerboseError<&str>>) -> Self {
        EvalError::VerboseError(i.to_string())
    }
}

impl From<TypeError> for EvalError {
    fn from(i: TypeError) -> Self {
        EvalError::TypeError(i.to_string())
    }
}

fn eval1(ctx: &mut Context, term: &Term) -> Result<Term, EvalError> {
    match term {
        Term::TmIf(if_term, then_term, else_term) => match *if_term.as_ref() {
            Term::TmTrue => Ok(then_term.as_ref().clone()),
            Term::TmFalse => Ok(else_term.as_ref().clone()),
            _ => {
                let if_term = eval(ctx, if_term.as_ref())?;
                Ok(Term::TmIf(
                    Box::new(if_term),
                    then_term.clone(),
                    else_term.clone(),
                ))
            }
        },
        Term::TmApp(left, right) => if left.as_ref() == &Term::TmAbs(name, typ, body) {},
        _ => Err(EvalError::NoRuleApplies),
    }
}

pub fn eval(ctx: &mut Context, term: &Term) -> Result<Term, EvalError> {
    let term = eval1(ctx, term)?;
    Ok(term)
}
