use crate::error::error_msgs;
use crate::COMMAND_SEPARATOR;
use crate::{CommandExpression, Compilable, Parser, ParserError};

pub static VALUE_UNDEFINED: char = '\x31';
pub static VALUE_BOOL: char = '\x32';
pub static VALUE_STRING: char = '\x33';
pub static VALUE_U8: char = '\x34';
pub static VALUE_I16: char = '\x35';
pub static VALUE_F32: char = '\x36';

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    U8(u8),
    I16(i16),
    F32(f32),
}

impl Value {
    /// Convert any to string, cannot be overwrite
    pub fn to_string_or_default(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::String(v) => v.clone(),
            Self::Bool(v) => format!("{v}"),
            Self::U8(v) => format!("{v}"),
            Self::I16(v) => format!("{v}"),
            Self::F32(v) => format!("{v}"),
        }
    }
}

impl Compilable for Value {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Null => format!("{VALUE_UNDEFINED}"),
            Self::Bool(v) => format!("{VALUE_BOOL}{}", if *v { '\x01' } else { '\x00' }),
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
            Self::U8(v) => format!("{VALUE_U8}{}", String::from_utf8_lossy(&[*v])),
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
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::String(v) => f.write_fmt(format_args!("{v:?}")),
            Self::Bool(v) => f.write_fmt(format_args!("{v:?}")),
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
    fn visit_u8(parser: &mut Parser) -> Option<u8> {
        let b = parser.consume()?;
        Some(b as u8)
    }

    fn visit_u16(parser: &mut Parser) -> Option<u16> {
        let b1 = parser.consume()? as u16;
        let b2 = parser.consume()? as u16;

        // b1 -> 0xFF00
        // b2 -> 0x00FF
        Some((b1 << 8) + b2)
    }

    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        let b = parser.consume().ok_or_else(|| {
            parser.error_corrupt(
                error_msgs::ERROR_INVALID_VALUE_DECL,
                "Can't get value kind",
                1,
            )
        })?;

        match b {
            b if b == VALUE_UNDEFINED => Ok(Value::Null),
            b if b == VALUE_BOOL => Ok(Value::Bool(
                parser.consume().ok_or_else(|| {
                    parser.error_corrupt(
                        error_msgs::ERROR_INVALID_VALUE_DECL,
                        format!("Can't get bool value"),
                        1,
                    )
                })? == '\x01',
            )),
            b if b == VALUE_U8 => {
                let b = Value::visit_u8(parser).ok_or_else(|| {
                    parser.error_corrupt(
                        error_msgs::ERROR_INVALID_VALUE_DECL,
                        "Can't get value on u8",
                        0,
                    )
                })?;

                Ok(Value::U8(b))
            }
            b if b == VALUE_I16 => {
                let sign = parser.consume().ok_or_else(|| {
                    parser.error_corrupt(
                        error_msgs::ERROR_INVALID_VALUE_DECL,
                        "Can't get sign on i16",
                        1,
                    )
                })? as u8;

                let sign: i16 = if sign == 1 { 1 } else { -1 };

                let num = Value::visit_u16(parser).ok_or_else(|| {
                    parser.error_corrupt(
                        error_msgs::ERROR_INVALID_VALUE_DECL,
                        "Can't get value on i16",
                        0,
                    )
                })? as i16;

                let num = sign * num;
                Ok(Value::I16(num))
            }
            // TODO: Rework this for safe parse and serialize.
            // Now it can't fail with numbers like `0xFF00`
            // because we use `0x00` as stop character and it
            // creates a conflict
            b if b == VALUE_F32 => {
                let mut carrier = vec![];
                loop {
                    let b = parser.consume().ok_or_else(|| {
                        parser.error_corrupt(
                            error_msgs::ERROR_INVALID_VALUE_DECL,
                            "Can't get value on f32",
                            1,
                        )
                    })? as u8;

                    if b == 0 {
                        let num =
                            String::from_utf8_lossy(&carrier)
                                .parse::<f32>()
                                .map_err(|err| {
                                    parser.error(
                                        error_msgs::ERROR_INVALID_VALUE_DECL,
                                        format!("Can't parse f32 value. \nCaused by {err}"),
                                        0,
                                    )
                                })?;
                        break Ok(Value::F32(num));
                    }

                    carrier.push(b);
                }
            }
            b if b == VALUE_STRING => {
                let mut carrier = vec![];
                loop {
                    let b = parser.consume().ok_or_else(|| {
                        parser.error_corrupt(
                            error_msgs::ERROR_INVALID_VALUE_DECL,
                            "Can't get value on string",
                            1,
                        )
                    })? as u8;

                    if b == 0 {
                        let s = String::from_utf8_lossy(&carrier);
                        break Ok(Value::String(s.to_string()));
                    }

                    carrier.push(b);

                    if b == 255 {
                        let b = parser.consume().ok_or_else(|| {
                            parser.error_corrupt(
                                error_msgs::ERROR_INVALID_VALUE_DECL,
                                "Can't get escaped value on string",
                                0,
                            )
                        })? as u8;

                        carrier.push(b);
                    }
                }
            }

            b => Err(parser.error(
                error_msgs::ERROR_UNKNOWN_VALUE_KIND,
                format!(
                    "Unrecognized byte: 0x{:02x}. Expected bytes: {:?}",
                    b as u8,
                    [
                        VALUE_UNDEFINED,
                        VALUE_U8,
                        VALUE_I16,
                        VALUE_F32,
                        VALUE_STRING
                    ]
                ),
                1,
            )),
        }
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
