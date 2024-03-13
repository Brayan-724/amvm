use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::runtime::Context;
use crate::{AmvmHeader, Command, Compilable, COMMAND_SEPARATOR};

#[derive(Debug, Clone)]
pub struct AmvmScope {
    pub header: Arc<AmvmHeader>,
    pub body: Rc<Vec<Command>>,
    pub context: Context,
}

impl AmvmScope {
    pub fn new(header: &Arc<AmvmHeader>, body: Vec<Command>, upper: Option<&Context>) -> Self {
        Self {
            header: Arc::clone(header),
            body: Rc::new(body),
            context: upper.map_or_else(|| Context::new(), |v| v.create_sub()),
        }
    }

    pub fn create_sub(&self, body: Vec<Command>) -> Self {
        Self {
            header: Arc::clone(&self.header),
            body: Rc::new(body),
            context: self.context.create_sub(),
        }
    }
}

impl Compilable for AmvmScope {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(format!(
            "{body}{COMMAND_SEPARATOR}",
            body = self.body.compile_bytecode()
        ))
    }
}

impl fmt::Display for AmvmScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{body:?}", body = self.body))
    }
}
