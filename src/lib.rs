mod runtime;
pub use runtime::Runtime;

mod parser;
pub use parser::Parser;

mod error;

#[cfg(feature = "useron")]
use serde::{Deserialize, Serialize};

pub static AMVM_HEADER: &'static str = "\x08\x48\x30"; // Arbitrary value for sign (0x0B4B30)
pub static COMMAND_SEPARATOR: char = '\0';

pub static CMD_DCLR_VAR: char = '\x01';
pub static CMD_ASGN_VAR: char = '\x0D';
pub static CMD_PUTS: char = '\x0E'; // <-- Last entry
pub static CMD_EVAL: char = '\x02';

pub static VAR_CONST: char = '\x0B';
pub static VAR_LET: char = '\x0C';

pub static EXPR_VALUE: char = '\x03';
pub static EXPR_VAR: char = '\x0A';
pub static EXPR_ADD: char = '\x09';

pub static VALUE_UNDEFINED: char = '\x04';
pub static VALUE_STRING: char = '\x05';
pub static VALUE_U8: char = '\x06';
pub static VALUE_I16: char = '\x07';
pub static VALUE_F32: char = '\x08';

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum Value {
    Undefined,
    String(String),
    U8(u8),
    I16(i16),
    F32(f32),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => f.write_str("undefined"),
            Self::String(v) => f.write_fmt(format_args!("\"{v}\"")),
            Self::U8(v) => f.write_fmt(format_args!("{v}u8")),
            Self::I16(v) => f.write_fmt(format_args!("{v}i16")),
            Self::F32(v) => f.write_fmt(format_args!("{v}f32")),
        }
    }
}

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value::String(self.into())
    }
}

impl Into<CommandExpression> for Value {
    fn into(self) -> CommandExpression {
        CommandExpression::Value(self)
    }
}

impl Into<Option<CommandExpression>> for Value {
    fn into(self) -> Option<CommandExpression> {
        Some(CommandExpression::Value(self))
    }
}

impl Into<Box<CommandExpression>> for Value {
    fn into(self) -> Box<CommandExpression> {
        CommandExpression::Value(self).into()
    }
}

impl Into<Option<Box<CommandExpression>>> for Value {
    fn into(self) -> Option<Box<CommandExpression>> {
        CommandExpression::Value(self).into()
    }
}

impl Value {
    pub fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Undefined => format!("{VALUE_UNDEFINED}{COMMAND_SEPARATOR}"),
            Self::String(s) => {
                // Safe bytecode strings
                let s = s
                    .replace(
                        |c: char| (c as u8) == b'\xFF',
                        &String::from_utf8_lossy(&[255, 255]),
                    )
                    .replace("\x00", &String::from_utf8_lossy(&[255, 00]));
                format!("{VALUE_STRING}{s}{COMMAND_SEPARATOR}")
            }
            Self::U8(v) => format!("{VALUE_U8}{}", String::from_utf8_lossy(&[v + 1])),
            Self::I16(v) => format!(
                "{VALUE_I16}{}",
                String::from_utf8_lossy(&[
                    if v.is_positive() { 1 } else { 0 },
                    (v.unsigned_abs() >> 8) as u8,
                    v.unsigned_abs() as u8,
                ]),
            ),
            Self::F32(v) => format!("{VALUE_F32}{v}{COMMAND_SEPARATOR}",),
        })
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum CommandExpression {
    Value(Value),
    Var(Value),
    Addition(Box<CommandExpression>, Box<CommandExpression>),
}

impl std::fmt::Display for CommandExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => v.fmt(f),
            Self::Var(v) => f.write_fmt(format_args!("Var({v})")),
            Self::Addition(a, b) => f.write_fmt(format_args!("{a} + {b}")),
        }
    }
}

impl Into<Option<Box<CommandExpression>>> for CommandExpression {
    fn into(self) -> Option<Box<CommandExpression>> {
        Some(Box::from(self))
    }
}

impl CommandExpression {
    pub fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Value(v) => v.compile_bytecode(),
            Self::Var(var) => Box::from(format!("{EXPR_VAR}{}", var.compile_bytecode())),
            Self::Addition(a, b) => Box::from(format!(
                "{EXPR_ADD}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            )),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum VariableKind {
    Const,
    Let,
}

impl std::fmt::Display for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableKind::Const => f.write_str("Const"),
            VariableKind::Let => f.write_str("Let"),
        }
    }
}

impl VariableKind {
    pub fn compile_bytecode(&self) -> char {
        match self {
            VariableKind::Const => VAR_CONST,
            VariableKind::Let => VAR_LET,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum Command {
    DeclareVariable {
        name: Value,
        value: CommandExpression,
        kind: VariableKind,
    },
    AssignVariable {
        name: Value,
        value: CommandExpression,
    },
    Puts {
        value: CommandExpression,
    },
    Evaluate {
        expr: CommandExpression,
    },
}

impl Command {
    pub fn get_kind(&self) -> char {
        match self {
            Self::DeclareVariable { .. } => CMD_DCLR_VAR,
            Self::AssignVariable { .. } => CMD_ASGN_VAR,
            Self::Puts { .. } => CMD_PUTS,
            Self::Evaluate { .. } => CMD_EVAL,
        }
    }
}
impl Compilable for Command {
    fn compile_bytecode(&self) -> Box<str> {
        match self {
            Self::DeclareVariable { name, value, kind } => {
                if !name.is_string() {
                    panic!("Variable name should be string");
                }
                let kind = kind.compile_bytecode();
                let name = name.compile_bytecode();
                let value = value.compile_bytecode();

                Box::from(format!("{CMD_DCLR_VAR}{kind}{name}{value}"))
            }
            Self::AssignVariable { name, value } => {
                if !name.is_string() {
                    panic!("Variable name should be string");
                }
                let name = name.compile_bytecode();
                let value = value.compile_bytecode();
                Box::from(format!("{CMD_ASGN_VAR}{name}{value}"))
            }
            Self::Puts { value } => {
                let value = value.compile_bytecode();
                Box::from(format!("{CMD_PUTS}{value}"))
            }
            _ => todo!(),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeclareVariable { name, value, kind } => {
                f.write_fmt(format_args!("DeclareVariable({kind}, {name}, {value})"))
            }
            Self::AssignVariable { name, value } => {
                f.write_fmt(format_args!("AssignVariable({name}, {value})"))
            }
            Self::Puts { value } => f.write_fmt(format_args!("Puts({value})")),
            Self::Evaluate { expr } => f.write_fmt(format_args!("Evaluate({expr})")),
        }
    }
}

pub trait Compilable {
    fn compile_bytecode(&self) -> Box<str>;
}

impl Compilable for [Command] {
    fn compile_bytecode(&self) -> Box<str> {
        self.iter()
            .map(|c| c.compile_bytecode().to_string())
            .collect::<Vec<String>>()
            .join("")
            .into()
    }
}
