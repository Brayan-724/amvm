pub mod aml3;
mod runtime;
pub use runtime::Runtime;

mod parser;
pub use parser::Parser;

mod tokens;
pub use tokens::{
    AmvmHeader, AmvmScope, AmvmTypeCasting, BinaryConditionKind, Command, CommandExpression,
    Program, Value,
};

pub use tokens::{CMD_ASGN_VAR, CMD_DCLR_VAR, CMD_EVAL, CMD_PUTS, CMD_SCOPE};
pub use tokens::{EXPR_ADD, EXPR_VALUE, EXPR_VAR};
pub use tokens::{VALUE_F32, VALUE_I16, VALUE_STRING, VALUE_U8, VALUE_UNDEFINED};

mod utils;
pub use utils::Compilable;

mod error;
pub use error::ParserError;

#[cfg(feature = "useron")]
use serde::{Deserialize, Serialize};

pub static AMVM_HEADER: &'static str = "\x08\x48\x30"; // Arbitrary value for sign (0x0B4B30)
pub static COMMAND_SEPARATOR: char = '\0';

pub static VAR_CONST: char = '\x01';
pub static VAR_LET: char = '\x02';

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
