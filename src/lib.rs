pub static AMVM_HEADER: &'static str = "\x08\x48\x30"; // Arbitrary value
pub static COMMAND_SEPARATOR: char = '\0';

pub static CMD_DCLR_VAR: char = '\x01';
pub static CMD_EVAL: char = '\x02';

pub static EXPR_VALUE: char = '\x03';

pub static VALUE_UNDEFINED: char = '\x04';
pub static VALUE_STRING: char = '\x05';
pub static VALUE_F32: char = '\x06';

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    String(String),
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
        })
    }
}

#[derive(Debug, Clone)]
pub enum CommandExpression {
    Value(Value),
}

impl CommandExpression {
    pub fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Value(v) => v.compile_bytecode(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    DeclareVariable {
        name: Box<str>,
        value: Option<Box<CommandExpression>>,
    },
    Evaluate {
        expr: CommandExpression,
    },
}

impl Command {
    pub fn get_kind(&self) -> char {
        match self {
            Self::DeclareVariable { .. } => CMD_DCLR_VAR,
            Self::Evaluate { .. } => CMD_EVAL,
        }
    }

    pub fn compile_bytecode(&self) -> Box<str> {
        let kind = Command::get_kind(&self);

        match self {
            Self::DeclareVariable { name, value } => {
                let name = Value::String(name.to_string()).compile_bytecode();
                let value = if let Some(value) = value {
                    value.compile_bytecode()
                } else {
                    CommandExpression::Value(Value::Undefined).compile_bytecode()
                };
                Box::from(format!("{kind}{name}{value}{COMMAND_SEPARATOR}"))
            }
            _ => todo!(),
        }
    }
}
