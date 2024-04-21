mod command;
pub use command::Command;
pub use command::{CMD_ASGN_VAR, CMD_DCLR_VAR, CMD_EVAL, CMD_PUTS, CMD_SCOPE};

mod expr;
pub use expr::{BinaryConditionKind, CommandExpression};
pub use expr::{EXPR_ADD, EXPR_VALUE, EXPR_VAR};

mod header;
pub use header::{AmvmHeader, AmvmTypeCasting};

mod program;
pub use program::Program;

mod scope;
pub use scope::AmvmScope;

mod r#type;
pub use r#type::{AmvmType, AmvmTypeDefinition};
pub use r#type::{TYPE_ANON, TYPE_CUSTOM, TYPE_STRING, TYPE_TUPLE, TYPE_U8, TYPE_UNION};

mod value;
pub use value::{Value, ValueObject};
pub use value::{
    VALUE_CHAR, VALUE_F32, VALUE_I16, VALUE_OBJECT, VALUE_STRING, VALUE_U8, VALUE_UNDEFINED,
};

#[macro_export(local_inner_macros)]
macro_rules! create_bytes {
    ($prev:expr; $name:ident $(, $($tail:tt)*)?) => {
        pub static $name: char = ($prev + 1u8) as char;
        $(create_bytes! {($prev + 1); $($tail)*})?
    };
}

// pub use create_bytes;
