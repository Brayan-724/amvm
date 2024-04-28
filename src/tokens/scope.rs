use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::{
    runtime::Context,
    tokens::{AmvmHeader, Command, COMMAND_SEPARATOR},
    Compilable,
};

#[derive(Debug, Clone)]
pub struct AmvmScope {
    pub header: Arc<AmvmHeader>,
    pub body: Rc<Vec<Command>>,
    pub context: Arc<RwLock<Context>>,
}

impl AmvmScope {
    pub fn new(
        header: &Arc<AmvmHeader>,
        body: Vec<Command>,
        upper: Option<Arc<RwLock<Context>>>,
    ) -> Self {
        Self {
            header: Arc::clone(header),
            body: Rc::new(body),
            context: Arc::new(RwLock::new(
                upper.map_or_else(|| Context::new(), |v| Context::create_sub(v)),
            )),
        }
    }

    pub fn create_sub(&self, body: Vec<Command>) -> Self {
        Self {
            header: Arc::clone(&self.header),
            body: Rc::new(body),
            context: Arc::new(RwLock::new(Context::create_sub(self.context.clone()))),
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
