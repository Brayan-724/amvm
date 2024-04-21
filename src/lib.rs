pub mod aml3;

mod macros;
// pub use macros::*;

mod runtime;
pub use runtime::Runtime;

pub mod parser;
pub use parser::{Parser, ParserResult};

mod tokens;
pub use tokens::{
    AmvmHeader, AmvmScope, AmvmType, AmvmTypeCasting, AmvmTypeDefinition, BinaryConditionKind,
    Command, CommandExpression, Program, Value, ValueObject,
};

pub use tokens::{CMD_ASGN_VAR, CMD_DCLR_VAR, CMD_EVAL, CMD_PUTS, CMD_SCOPE};
pub use tokens::{EXPR_ADD, EXPR_VALUE, EXPR_VAR};
pub use tokens::{TYPE_ANON, TYPE_CUSTOM, TYPE_STRING, TYPE_TUPLE, TYPE_U8, TYPE_UNION};
pub use tokens::{
    VALUE_CHAR, VALUE_F32, VALUE_I16, VALUE_OBJECT, VALUE_STRING, VALUE_U8, VALUE_UNDEFINED,
};

mod utils;
pub use utils::Compilable;

mod error;
pub use error::ParserError;

#[cfg(feature = "useron")]
use serde::{Deserialize, Serialize};

pub static AMVM_HEADER: &'static str = "\x08\x48\x30"; // Arbitrary value for sign (0x0B4B30)
pub static COMMAND_SEPARATOR: char = '\0';

pub static VAR_CONST: char = '\x01';
pub static VAR_MUT: char = '\x02';
pub static VAR_LET: char = '\x03';
pub static VAR_VAR: char = '\x04';

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum VariableKind {
    Const,
    Mut,
    Let,
    Var,
}

impl std::fmt::Display for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableKind::Const => f.write_str("Const"),
            VariableKind::Mut => f.write_str("Mut"),
            VariableKind::Let => f.write_str("Let"),
            VariableKind::Var => f.write_str("Var"),
        }
    }
}

impl VariableKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "const" => Some(Self::Const),
            "mut" => Some(Self::Mut),
            "let" => Some(Self::Let),
            "var" => Some(Self::Var),
            _ => None,
        }
    }
    pub fn compile_bytecode(&self) -> char {
        match self {
            VariableKind::Const => VAR_CONST,
            VariableKind::Mut => VAR_MUT,
            VariableKind::Let => VAR_LET,
            VariableKind::Var => VAR_VAR,
        }
    }
}
