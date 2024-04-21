use std::collections::HashMap;

use crate::parser::{self, anychar, ParserResult};
use crate::{AmvmType, COMMAND_SEPARATOR};
use crate::{CommandExpression, Compilable, Parser};

pub static VALUE_UNDEFINED: char = '\x30';
pub static VALUE_BOOL: char = '\x31';
pub static VALUE_STRING: char = '\x32';
pub static VALUE_U8: char = '\x33';
pub static VALUE_I16: char = '\x34';
pub static VALUE_F32: char = '\x35';
pub static VALUE_OBJECT: char = '\x36';
pub static VALUE_CHAR: char = '\x37';

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum ValueObject {
    Native(*mut u32),
    Instance(AmvmType, HashMap<String, Value>),
    PropertyMap(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum Value {
    Null,
    String(String),
    Char(char),
    Bool(bool),
    U8(u8),
    I16(i16),
    F32(f32),
    Object(ValueObject),
}

impl ValueObject {
    pub fn to_native_mutable<T>(&self) -> Option<&mut T> {
        if let Self::Native(ptr) = self {
            unsafe { (*ptr as *mut T).as_mut() }
        } else {
            None
        }
    }

    pub fn into_native_boxed<T>(&self) -> Option<Box<T>> {
        if let Self::Native(ptr) = self {
            Some(unsafe { Box::from_raw(*ptr as *mut T) })
        } else {
            None
        }
    }
}

impl Value {
    /// Convert any to string, cannot be overwrite
    pub fn to_string_or_default(&self) -> String {
        match self {
            Self::Null => String::from("null"),
            Self::Char(v) => v.to_string(),
            Self::String(v) => v.clone(),
            Self::Bool(v) => format!("{v}"),
            Self::U8(v) => format!("{v}"),
            Self::I16(v) => format!("{v}"),
            Self::F32(v) => format!("{v}"),
            Self::Object(v) => match v {
                ValueObject::Native(ref v) => format!("[Object 0x{:08x}]", *v as u32),
                _ => todo!(),
            },
        }
    }

    pub fn compile_string(string: impl AsRef<str>) -> String {
        let string = string.as_ref();
        let string_len = string.len() as u8 as char;

        format!("{string_len}{string}")
    }

    pub fn visit_string<'a>(parser: Parser<'a>) -> ParserResult<'a, &str> {
        let (parser, string_len) = parser::anychar(parser).map_err(|_a: parser::Err<()>| {
            parser.error(
                parser::VerboseErrorKind::Context("Can't get string length"),
                true,
            )
        })?;
        let string_len = string_len as u8;

        tracing::trace!(?string_len);

        let (string, parser) = parser::take(string_len)(parser)?;
        tracing::trace!(string = string.value);

        Ok((parser, string.value))
    }

    pub fn as_object(&self) -> Option<&ValueObject> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Compilable for Value {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Null => format!("{VALUE_UNDEFINED}"),
            Self::Bool(v) => format!("{VALUE_BOOL}{}", if *v { '\x01' } else { '\x00' }),
            Self::Char(v) => format!("{VALUE_CHAR}{v}"),
            Self::String(string) => {
                let string_len = string.len() as u8 as char;

                format!("{VALUE_STRING}{string_len}{string}")
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
            Self::Object(_) => format!("{VALUE_OBJECT}"),
        })
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Char(v) => f.write_fmt(format_args!("{v:?}")),
            Self::String(v) => f.write_fmt(format_args!("{v:?}")),
            Self::Bool(v) => f.write_fmt(format_args!("{v:?}")),
            Self::U8(v) => f.write_fmt(format_args!("{v}u8")),
            Self::I16(v) => f.write_fmt(format_args!("{v}i16")),
            Self::F32(v) => f.write_fmt(format_args!("{v}f32")),
            Self::Object(_) => f.write_str("[Native Object]"),
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

    #[tracing::instrument("visit_value", fields(at = parser.pointer_position(), parser = tracing::field::Empty), level = tracing::Level::TRACE)]
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, b) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected value kind"))?;

        let (parser, value) = match b {
            b if b == VALUE_UNDEFINED => {
                let _tracing_span = tracing::trace_span!("null");
                let _tracing_span = _tracing_span.enter();

                (parser, Value::Null)
            }
            b if b == VALUE_BOOL => {
                let _tracing_span = tracing::trace_span!("bool");
                let _tracing_span = _tracing_span.enter();

                let (parser, value) = anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected boolean value"))?;
                (parser, Value::Bool(value == '\x01'))
            }
            b if b == VALUE_U8 => {
                let _tracing_span = tracing::trace_span!("u8");
                let _tracing_span = _tracing_span.enter();

                let (parser, b) = Value::visit_u8(parser)?;
                (parser, Value::U8(b))
            }
            b if b == VALUE_I16 => {
                let _tracing_span = tracing::trace_span!("i16");
                let _tracing_span = _tracing_span.enter();

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
                let _tracing_span = tracing::trace_span!("f32");
                let _tracing_span = _tracing_span.enter();

                let mut carrier = vec![];
                loop {
                    let (parser, b) = anychar(parser)?;
                    // "Can't get value on f32",

                    if b == '\x00' {
                        let num =
                            String::from_utf8_lossy(&carrier)
                                .parse::<f32>()
                                .map_err(|_| {
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
                let _tracing_span = tracing::trace_span!("string");
                let _tracing_span = _tracing_span.enter();

                let (parser, string_len) =
                    parser::anychar(parser).map_err(|_a: parser::Err<()>| {
                        parser.error(
                            parser::VerboseErrorKind::Context("Can't get string length"),
                            true,
                        )
                    })?;
                let string_len = string_len as u8;

                tracing::trace!(?string_len);

                let (string, parser) = parser::take(string_len)(parser)?;
                tracing::trace!(string = string.value);

                (parser, Value::String(string.value.to_owned()))
            }
            b if b == VALUE_CHAR => {
                let _tracing_span = tracing::trace_span!("char");
                let _tracing_span = _tracing_span.enter();

                let (parser, char) = parser::anychar(parser).map_err(|_a: parser::Err<()>| {
                    parser.error(
                        parser::VerboseErrorKind::Context("Expected char value"),
                        true,
                    )
                })?;

                (parser, Value::Char(char))
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
