use std::fmt::Display;
use std::fmt::Formatter;

use untyped_arith::parser::parse;
use untyped_arith::parser::Term;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    Boolean,
    Numeric,
}

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

fn term_type(term: &Term) -> Result<Type> {
    let term_type = match term {
        Term::TmTrue => Type::Boolean,
        Term::TmFalse => Type::Boolean,
        Term::TmZero => Type::Numeric,
        Term::TmSucc(term) => match term_type(term)? {
            Type::Numeric => Type::Numeric,
            _ => {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            }
        },
        Term::TmPred(term) => match term_type(term)? {
            Type::Numeric => Type::Numeric,
            _ => {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            }
        },
        Term::TmIsZero(term) => match term_type(term)? {
            Type::Numeric => Type::Numeric,
            _ => {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            }
        },
        Term::TmIf(cond_term, then_term, else_term) => {
            let cond_type = term_type(cond_term.as_ref())?;
            match cond_type {
                Type::Boolean => {
                    let then_type = term_type(then_term.as_ref())?;
                    let else_type = term_type(else_term.as_ref())?;
                    if then_type != else_type {
                        return Err(Error {
                            msg: format!(
                                "then term {:?} mismatch with else term {:?}",
                                then_term, else_term
                            ),
                        });
                    }

                    then_type
                }
                _ => {
                    return Err(Error {
                        msg: format!("term {:?} MUST be Boolean", cond_term),
                    });
                }
            }
        }
    };
    Ok(term_type)
}

pub fn eval_term(term: &Term) -> Result<Value> {
    let value = match term {
        Term::TmTrue => Value::Boolean(true),
        Term::TmFalse => Value::Boolean(false),
        Term::TmZero => Value::Numeric(0),
        Term::TmSucc(term) => {
            let value = if let Value::Numeric(number) = eval_term(term.as_ref())? {
                Value::Numeric(number + 1)
            } else {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            };
            value
        }
        Term::TmPred(term) => {
            let value = if let Value::Numeric(number) = eval_term(term.as_ref())? {
                Value::Numeric(number - 1)
            } else {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            };
            value
        }
        Term::TmIsZero(term) => {
            if term_type(term)? != Type::Numeric {
                return Err(Error {
                    msg: format!("term {:?} MUST be Numeric", term),
                });
            }
            Value::Boolean(term.is_zero())
        }
        Term::TmIf(cond_term, then_term, else_term) => {
            let _ = term_type(term)?;
            if let Value::Boolean(cond) = eval_term(cond_term.as_ref())? {
                if cond {
                    eval_term(&then_term.as_ref())?
                } else {
                    eval_term(&else_term.as_ref())?
                }
            } else {
                return Err(Error {
                    msg: format!("term {:?} MUST be Boolean", cond_term),
                });
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
        assert!(eval("iszero(false);").is_err());
        assert_eq!(eval("if false then 10 else 20;")?, Value::Numeric(20));
        assert!(eval("if 9 then 10 else 20;").is_err());
        assert!(eval("if true then false else 20;").is_err());
        assert_eq!(
            eval("if false then true else false;")?,
            Value::Boolean(false)
        );
        Ok(())
    }
}
