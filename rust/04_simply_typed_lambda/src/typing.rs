use std::fmt::{self, Formatter};

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
