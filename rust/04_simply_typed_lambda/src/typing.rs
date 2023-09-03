use std::fmt::{self, Formatter};

use crate::{context::Context, parser::Term};

#[derive(Debug)]
pub enum TypeError {
    ParameterTypeMismatch,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    Boolean,
    Number,
}

/*
pub fn type_of(ctx: &Context, term: &Term) -> Result<Type, TypeError> {
    match term {
        Term::TmTrue | Term::TmFalse => Ok(Type::Boolean),
        Term::TmZero => Ok(Type::Number),
        Term::TmSucc(var) => {
            if let Ok(Type::Number) = type_of(ctx, var.as_ref()) {
                Ok(Type::Number)
            } else {
                Err(TypeError::ParameterTypeMismatch)
            }
        }
    }
}
*/
