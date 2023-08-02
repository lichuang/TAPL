use core::panic;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::parser::parse;
use crate::parser::Term;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Boolean(bool),
    Numeric(u8),
}

#[derive(Clone, Debug)]
pub struct Error {
    msg: String,
}

impl From<nom::Err<nom::error::VerboseError<&str>>> for Error {
    fn from(error: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        Error {
            msg: format!("nom parser error: {}", error.to_string()),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)?;
        Ok(())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn eval_term(term: &Term) -> Result<Value> {
    let value = match term {
        Term::TmTrue => Value::Boolean(true),
        Term::TmFalse => Value::Boolean(false),
        Term::TmZero => Value::Numeric(0),
        Term::TmSucc(term) => {
            let value = if let Value::Numeric(number) = eval_term(term.as_ref())? {
                Value::Numeric(number + 1)
            } else {
                panic!("succ MUST operate with Numeric");
            };
            value
        }
        Term::TmPred(term) => {
            let value = if let Value::Numeric(number) = eval_term(term.as_ref())? {
                Value::Numeric(number - 1)
            } else {
                panic!("pred MUST operate with Numeric");
            };
            value
        }
        Term::TmIsZero(term) => Value::Boolean(term.is_zero()),
        Term::TmIf(cond_term, then_term, else_term) => {
            if let Value::Boolean(cond) = eval_term(cond_term.as_ref())? {
                if cond {
                    eval_term(&then_term.as_ref())?
                } else {
                    eval_term(&else_term.as_ref())?
                }
            } else {
                panic!("if condition MUST operate with Boolean");
            }
        }
    };
    Ok(value)
}

pub fn eval(input: &str) -> Result<Value> {
    let term = parse(input)?;
    // assert has no input string left
    assert!(term.0.is_empty());

    eval_term(&term.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() -> Result<()> {
        assert_eq!(eval("true;")?, Value::Boolean(true));
        assert_eq!(eval("succ(2);")?, Value::Numeric(3));
        assert_eq!(eval("iszero(2);")?, Value::Boolean(false));
        assert_eq!(eval("if false then 10 else 20;")?, Value::Numeric(20));
        Ok(())
    }
}
