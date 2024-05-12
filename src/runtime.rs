use std::io::Write;
use std::rc::Rc;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use crate::tokens::{AmvmHeader, AmvmScope, AmvmTypeDefinition, Command, Value};

mod commands;
pub mod core;
mod error;
mod expr;
mod result;
mod scope;
pub mod variable;

pub use error::AmvmError;
pub use expr::AmvmExprResult;
pub use result::{AmvmPropagate, AmvmResult};
pub use variable::AmvmVariable;

const PREV_MAX: usize = u8::MAX as usize;

#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<String, AmvmVariable>,
    prev: Vec<AmvmExprResult>,

    structs: HashMap<String, AmvmTypeDefinition>,

    parent: Option<Arc<Mutex<Context>>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Default::default(),
            structs: Default::default(),
            prev: Vec::with_capacity(PREV_MAX),
            parent: None,
        }
    }

    pub fn create_sub(this: Arc<Mutex<Context>>) -> Self {
        Self {
            variables: Default::default(),
            structs: Default::default(),
            prev: Vec::with_capacity(PREV_MAX),
            parent: Some(Arc::clone(&this)),
        }
    }

    pub fn pop_prev(&mut self) -> Option<AmvmExprResult> {
        self.prev.pop()
    }

    pub fn push_prev(&mut self, v: AmvmExprResult) {
        self.prev.push(v)
    }

    pub fn push_prev_value(&mut self, v: Value) {
        self.push_prev(AmvmExprResult::from(v))
    }

    pub fn get_variable(&self, name: &String) -> AmvmVariable {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .map(|p| p.lock().unwrap().get_variable(name))
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
    pub fn new(filename: Box<str>, header: AmvmHeader, ast: Vec<Command>) -> Self {
        Self {
            scope: AmvmScope::new(filename, &Rc::new(header), ast, None),
        }
    }

    fn registry_base_types(&self) {
        let structs = &mut self.scope.context.lock().unwrap().structs;
        structs.insert("Iterator".to_owned(), core::amvm_iterator_type());
    }

    pub fn run(&mut self) -> AmvmResult {
        self.registry_base_types();

        for cmd in self.scope.body.clone().iter() {
            commands::eval(&mut self.scope, cmd)?;
        }

        Ok(Value::Null)
    }
}

pub fn get_backtrace() -> String {
    let backtrace = format!("{}", std::backtrace::Backtrace::capture());
    backtrace
        .split('\n')
        .filter(|l| {
            !l.contains("at /rustc/")
                && !l.contains(": std::")
                && !l.contains(": core::")
                && !l.contains(": _")
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
