mod commands;
mod error;
mod expr;
mod result;
mod scope;
pub mod variable;

pub use error::AmvmError;
pub use result::{AmvmPropagate, AmvmResult};

use crate::{AmvmHeader, AmvmScope, Command, Value};
use std::io::Write;
use std::{collections::HashMap, sync::Arc};

use self::variable::AmvmVariable;

#[derive(Debug, Clone)]
pub struct Context {
    // TODO: Create struct for variables that contains a reference
    // counter, this will help garbage collection.
    variables: HashMap<String, AmvmVariable>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn create_sub(&self) -> Self {
        Self {
            variables: self.variables.clone(),
        }
    }

    pub fn get_variable(&self, name: &String) -> &AmvmVariable {
        self.variables.get(name).unwrap_or_else(|| {
            let backtrace = format!("{}", std::backtrace::Backtrace::capture());
            let backtrace = backtrace
                .split("\n")
                .filter(|l| {
                    !l.contains("at /rustc/")
                        && !l.contains(": std::")
                        && !l.contains(": core::")
                        && !l.contains(": _")
                })
                .collect::<Vec<&str>>()
                .join("\n");

            let _ = std::io::stdout().lock().flush();

            eprintln!(
                "\n\x1b[31mERROR: Variable {name:?} is not defined.\n\x1b[2m{backtrace}\x1b[0m"
            );
            std::process::exit(1)
        })
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {
    scope: AmvmScope,
}

impl Runtime {
    pub fn new(header: AmvmHeader, ast: Vec<Command>) -> Self {
        Self {
            scope: AmvmScope::new(&Arc::new(header), ast, None),
        }
    }

    pub fn run(&mut self) -> AmvmResult {
        for cmd in self.scope.body.clone().iter() {
            commands::eval(&mut self.scope, cmd)?;
        }

        Ok(Value::Null)
    }
}
