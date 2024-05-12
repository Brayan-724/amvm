use std::rc::Rc;
use std::{error, fmt};

use crate::tokens::AmvmMeta;

#[derive(Debug, Clone)]
pub enum AmvmError {
    Other(Vec<Rc<AmvmMeta>>, &'static str),
}

impl fmt::Display for AmvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(meta, ctx) => {
                writeln!(f, "\x1b[1;31merror:\x1b[0;1m {ctx}\x1b[0m")?;

                let mut has_alternatives = false;

                let debug_ir = std::env::var("AMVM_IR_DEBUG")
                    .map(|x| x != "0" && x != "false")
                    .unwrap_or(false);

                for meta in meta.iter() {
                    if let Some(alternative) = meta.alternative.as_ref() {
                        has_alternatives = true;
                        write!(f, "{}", alternative.display(false, false))?;

                        if debug_ir {
                            write!(f, "{}", meta.display(true, true))?;
                        }
                    } else {
                        write!(f, "{}", meta.display(true, false))?;
                    }
                }

                if !debug_ir && has_alternatives {
                    writeln!(
                        f,
                        "\x1b[31mAlternative code detected, use AMVM_IR_DEBUG=1 to see original.\x1b[0m"
                    )?;
                }

                if meta.is_empty() {
                    writeln!(f, "\x1b[31mNo backtrace. Available through AML3_DEBUG=1 when compiling from aml3\x1b[0m")?;
                }

                Ok(())
            }
        }
    }
}

impl error::Error for AmvmError {}
