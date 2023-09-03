use crate::{
    context::Context,
    parser::Term,
    typing::{Type, TypeError},
};

use nom::error::VerboseError;

pub enum EvalError {
    VerboseError(String),
    TypeError(String),
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

/*
pub fn eval(ctx: &mut Context, input: &str) -> Result<(Term, Type), EvalError> {
    let (i, term) = parse(input)?;
    assert!(i.is_empty());
    let typ = type_of(ctx, &term)?;

    Ok((term, typ))
}
*/
