mod commands;
mod error;
mod expr;
mod result;
mod scope;
pub mod variable;

pub use error::AmvmError;
pub use result::{AmvmPropagate, AmvmResult};

use crate::{AmvmHeader, AmvmScope, AmvmTypeDefinition, Command, Value};
use std::io::Write;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use self::expr::AmvmExprResult;
use self::variable::AmvmVariable;

#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<String, AmvmVariable>,
    prev: Arc<RwLock<Option<AmvmExprResult>>>,

    structs: HashMap<String, AmvmTypeDefinition>,

    parent: Option<Arc<RwLock<Context>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Default::default(),
            structs: Default::default(),
            prev: Arc::new(RwLock::new(None)),
            parent: None,
        }
    }

    pub fn create_sub(this: Arc<RwLock<Context>>) -> Self {
        Self {
            variables: Default::default(),
            structs: Default::default(),
            prev: Arc::new(RwLock::new(None)),
            parent: Some(Arc::clone(&this)),
        }
    }

    pub fn get_prev(&self) -> Option<AmvmExprResult> {
        self.prev.write().unwrap().take()
    }

    pub fn set_prev(&self, v: AmvmExprResult) {
        *self.prev.write().unwrap() = Some(v);
    }

    pub fn set_prev_value(&self, v: Value) {
        self.set_prev(AmvmExprResult::from(v))
    }

    pub fn get_variable(&self, name: &String) -> AmvmVariable {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .map(|p| p.read().unwrap().get_variable(name))
            })
            .unwrap_or_else(|| {
                let _ = std::io::stdout().lock().flush();

                let backtrace = get_backtrace();

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

pub fn get_backtrace() -> String {
    let backtrace = format!("{}", std::backtrace::Backtrace::capture());
    backtrace
        .split("\n")
        .filter(|l| {
            !l.contains("at /rustc/")
                && !l.contains(": std::")
                && !l.contains(": core::")
                && !l.contains(": _")
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
