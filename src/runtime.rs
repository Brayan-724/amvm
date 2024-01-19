use crate::{Command, CommandExpression, Value};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    variables: HashMap<String, Value>,
}
impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {
    ast: Vec<Command>,
    pointer: usize,
    context: Arc<RwLock<Context>>,
}

impl Runtime {
    pub fn new(ast: Vec<Command>) -> Self {
        Self {
            ast,
            pointer: 0,
            context: Arc::new(RwLock::new(Context::new())),
        }
    }

    pub fn eval(&mut self, expr: &CommandExpression) -> Value {
        match expr {
            CommandExpression::Value(v) => v.clone(),
            CommandExpression::Addition(a, b) => {
                let a = self.eval(a);
                let b = self.eval(b);

                match (a, b) {
                    (Value::Undefined, Value::Undefined) => Value::Undefined,
                    (Value::U8(a), Value::U8(b)) => Value::U8(a + b),
                    (Value::I16(a), Value::I16(b)) => Value::I16(a + b),
                    (Value::F32(a), Value::F32(b)) => Value::F32(a + b),
                    (Value::String(a), Value::String(b)) => Value::String(format!("{a}{b}")),
                    _ => Value::Undefined,
                }
            }
        }
    }

    pub fn step(&mut self) {
        let cmd = &self.ast[self.pointer].clone();
        self.pointer += 1;

        match cmd {
            Command::DeclareVariable { name, value } => {
                let name = name.to_string();
                let value = if let Some(value) = value {
                    self.eval(value.as_ref())
                } else {
                    Value::Undefined
                };
                let mut context = self.context.write().unwrap();
                context.variables.insert(name, value);
            }
            Command::Evaluate { expr } => {
                self.eval(expr);
            }
        }
    }
}
