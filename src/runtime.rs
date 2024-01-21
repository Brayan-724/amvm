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
            CommandExpression::Var(var) => {
                let var = var.as_string().expect("Variable name should be string");
                self.context
                    .read()
                    .unwrap()
                    .variables
                    .get(var)
                    .cloned()
                    .unwrap_or(Value::Undefined)
            }
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
            Command::DeclareVariable { name, value, .. } => {
                let name = name
                    .as_string()
                    .expect("Variable name should be string")
                    .clone();
                let value = if let Some(value) = value {
                    self.eval(value)
                } else {
                    Value::Undefined
                };
                let mut context = self.context.write().unwrap();
                context.variables.insert(name, value);
            }
            Command::AssignVariable { name, value } => {
                let name = name
                    .as_string()
                    .expect("Variable name should be string")
                    .clone();
                let value = self.eval(value);
                let mut context = self.context.write().unwrap();
                context.variables.insert(name, value);
            }
            Command::Puts { value } => {
                let value = self.eval(value);
                match value {
                    Value::Undefined => print!("undefined"),
                    Value::String(v) => print!("{v}"),
                    Value::U8(v) => print!("{v}"),
                    Value::I16(v) => print!("{v}"),
                    Value::F32(v) => print!("{v}"),
                }
            }
            Command::Evaluate { expr } => {
                self.eval(expr);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.pointer >= self.ast.len() {
                break;
            }

            self.step();
        }
    }
}
