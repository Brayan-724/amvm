use crate::parser::{self, anychar, ParserResult};
use crate::COMMAND_SEPARATOR;
use crate::{CommandExpression, Compilable, Parser};

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
    fn visit_u8<'a>(parser: Parser<'a>) -> ParserResult<'a, u8> {
        let (parser, value) = anychar(parser)?;

        Ok((parser, value as u8))
    }

    fn visit_u16<'a>(parser: Parser<'a>) -> ParserResult<'a, u16> {
        let (parser, b1) = anychar(parser)?;
        let (parser, b2) = anychar(parser)?;

        // b1 -> 0xFF00
        // b2 -> 0x00FF
        Ok((parser, ((b1 as u16) << 8) + b2 as u16))
    }

    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, b) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected value kind"))?;

        let (parser, value) = match b {
            b if b == VALUE_UNDEFINED => (parser, Value::Null),
            b if b == VALUE_BOOL => {
                let (parser, value) = anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected boolean value"))?;
                (parser, Value::Bool(value == '\x01'))
            }
            b if b == VALUE_U8 => {
                let (parser, b) = Value::visit_u8(parser)?;
                (parser, Value::U8(b))
            }
            b if b == VALUE_I16 => {
                let (parser, sign) = anychar(parser)?;
                let sign: i16 = if sign == '\x01' { 1 } else { -1 };

                let (parser, num) = Value::visit_u16(parser)?;

                let num = sign * num as i16;
                (parser, Value::I16(num))
            }
            // TODO: Rework this for safe parse and serialize.
            // Now it can't fail with numbers like `0xFF00`
            // because we use `0x00` as stop character and it
            // creates a conflict
            b if b == VALUE_F32 => {
                let mut carrier = vec![];
                loop {
                    let (parser, b) = anychar(parser)?;
                    // "Can't get value on f32",

                    if b == '\x00' {
                        let num =
                            String::from_utf8_lossy(&carrier)
                                .parse::<f32>()
                                .map_err(|err| {
                                    // format!("Can't parse f32 value. \nCaused by {err}"),
                                    parser::Err::Failure(parser::VerboseError {
                                        errors: vec![(
                                            parser,
                                            parser::VerboseErrorKind::Context(
                                                "Can't parser f32 value",
                                            ),
                                        )],
                                    })
                                })?;
                        break (parser, Value::F32(num));
                    }

                    carrier.push(b as u8);
                }
            }
            b if b == VALUE_STRING => {
                let mut carrier = vec![];
                let mut parser = parser;
                loop {
                    // "Can't get value on string",
                    let (_parser, _b) = parser::anychar(parser)?;
                    let b = _b as u8;
                    parser = _parser;

                    if b == 0 {
                        let s = String::from_utf8_lossy(&carrier);
                        break (parser, Value::String(s.to_string()));
                    }

                    if b == 255 {
                        // "Can't get escaped value on string",
                        let (_parser, _b) = anychar(parser)?;
                        let b = _b as u8;
                        parser = _parser;
                        carrier.push(b as u8);
                    } else {
                        carrier.push(b as u8);
                    }
                }
            }

            b => {
                return Err(parser::Err::Failure(parser::VerboseError {
                    errors: vec![(parser, parser::VerboseErrorKind::Char(b))],
                }))
            }
        };

        Ok((parser, value))
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
