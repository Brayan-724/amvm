mod command;
pub use command::Command;
pub use command::{CMD_ASGN_VAR, CMD_DCLR_VAR, CMD_PUTS, CMD_SCOPE};

mod expr;
pub use expr::{BinaryKind, CommandExpression};
pub use expr::{EXPR_VALUE, EXPR_VAR};

mod header;
pub use header::{AmvmHeader, AmvmTypeCasting};

mod program;
pub use program::Program;

mod scope;
pub use scope::{AmvmMeta, AmvmScope};

mod r#type;
pub use r#type::{AmvmPrimitiveType, AmvmType, AmvmTypeDefinition};
pub use r#type::{TYPE_ANON, TYPE_CUSTOM, TYPE_STRING, TYPE_TUPLE, TYPE_U8, TYPE_UNION};

mod value;
pub use value::{Value, ValueFun, ValueObject};
pub use value::{
    VALUE_CHAR, VALUE_F32, VALUE_FUN, VALUE_I16, VALUE_OBJECT, VALUE_STRING, VALUE_U8,
    VALUE_UNDEFINED,
};

pub static AMVM_HEADER: &str = "\x08\x48\x30"; // Arbitrary value for sign (0x0B4B30)
pub static COMMAND_SEPARATOR: char = '\0';

pub static VAR_CONST: char = '\x01';
pub static VAR_MUT: char = '\x02';
pub static VAR_LET: char = '\x03';
pub static VAR_VAR: char = '\x04';

#[macro_export(local_inner_macros)]
macro_rules! create_bytes {
    ($prev:expr; #[doc = $_:literal] $($tail:tt)*) => {
        create_bytes! {$prev; $($tail)*}
    };
    ($prev:expr; $name:ident $(, $($tail:tt)*)?) => {
        pub const $name: char = ($prev + 1u8) as char;
        $(create_bytes! {($prev + 1); $($tail)*})?
    };
}

#[cfg(feature = "useron")]
use serde::{Deserialize, Serialize};

use crate::Compilable;

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum VariableKind {
    Const,
    Mut,
    Let,
    Var,
}

impl std::cmp::PartialOrd for VariableKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Const, Self::Const) => Some(std::cmp::Ordering::Equal),
            (_, Self::Const) => Some(std::cmp::Ordering::Greater),
            (Self::Const, _) => Some(std::cmp::Ordering::Less),

            (Self::Let, Self::Let) => Some(std::cmp::Ordering::Equal),
            (_, Self::Let) => Some(std::cmp::Ordering::Greater),
            (Self::Let, _) => Some(std::cmp::Ordering::Less),

            (Self::Mut, Self::Mut) => Some(std::cmp::Ordering::Equal),
            (_, Self::Mut) => Some(std::cmp::Ordering::Greater),
            (Self::Mut, _) => Some(std::cmp::Ordering::Less),

            (Self::Var, Self::Var) => Some(std::cmp::Ordering::Equal),
        }
    }
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
}

impl Compilable for VariableKind {
    fn compile_bytecode(&self, mut buffer: String) -> crate::CompileResult {
        use std::fmt::Write;

        _ = buffer.write_char(match self {
            VariableKind::Const => VAR_CONST,
            VariableKind::Mut => VAR_MUT,
            VariableKind::Let => VAR_LET,
            VariableKind::Var => VAR_VAR,
        });

        Ok(buffer)
    }
}
