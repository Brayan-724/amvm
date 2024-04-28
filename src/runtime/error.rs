use std::{error, fmt};

#[derive(Debug, Clone)]
pub enum AmvmError {
    Other(&'static str),
}

impl fmt::Display for AmvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(t) => f.write_str(t),
        }
    }
}

impl error::Error for AmvmError {}
