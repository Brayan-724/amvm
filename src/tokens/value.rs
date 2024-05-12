use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::runtime::{AmvmResult, AmvmVariable};
use crate::utils::CompileResult;
use crate::{
    create_bytes,
    parser::{self, Parser, ParserResult},
    tokens::{AmvmType, Command, CommandExpression, COMMAND_SEPARATOR},
    Compilable,
};

use super::{AmvmScope, VariableKind};

create_bytes! {0x30;
    VALUE_UNDEFINED,
    VALUE_BOOL,
    VALUE_STRING,
    VALUE_U8,
    VALUE_I16,
    VALUE_F32,
    VALUE_OBJECT,
    VALUE_CHAR,
    VALUE_FUN
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum ValueObject {
    Native(*mut u32),
    Instance(AmvmType, HashMap<String, Arc<RwLock<Value>>>),
    PropertyMap(HashMap<String, Arc<RwLock<Value>>>),
}

#[derive(Clone)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum ValueFun {
    Native(
        Vec<(Box<str>, VariableKind, AmvmType)>,
        AmvmType,
        Rc<RefCell<dyn FnMut(&mut AmvmScope) -> AmvmResult>>,
    ),
    Const(
        Vec<(Box<str>, VariableKind, AmvmType)>,
        AmvmType,
        Vec<Command>,
    ),
    Mutable(
        Vec<(Box<str>, VariableKind, AmvmType)>,
        AmvmType,
        Vec<Command>,
    ),
}

impl std::fmt::Debug for ValueFun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(args, ret, _) => f.debug_tuple("Native").field(args).field(ret).finish(),
            Self::Const(args, ret, body) => f
                .debug_tuple("Const")
                .field(args)
                .field(ret)
                .field(&body.len())
                .finish(),
            Self::Mutable(args, ret, body) => f
                .debug_tuple("Mutable")
                .field(args)
                .field(ret)
                .field(&body.len())
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "useron", derive(Serialize, Deserialize))]
pub enum Value {
    Null,

    Char(char),
    Bool(bool),
    I16(i16),
    F32(f32),
    Fun(ValueFun),
    Object(ValueObject),
    Ref(AmvmVariable),
    String(String),
    U8(u8),
}

impl ValueObject {
    pub fn to_native_mutable<T>(&self) -> Option<&mut T> {
        if let Self::Native(ptr) = self {
            unsafe { Some(&mut *(*ptr as *mut T)) }
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
            Self::Fun(v) => match v {
                ValueFun::Native(ref args, ret, _)
                | ValueFun::Const(ref args, ret, _)
                | ValueFun::Mutable(ref args, ret, _) => {
                    format!(
                        "[Function ({args}) {ret}]",
                        args = args
                            .iter()
                            .map(|a| format!(
                                "{kind} {name}: {ty}",
                                name = a.0,
                                kind = a.1,
                                ty = a.2.flat_name()
                            ))
                            .collect::<Vec<String>>()
                            .join(", "),
                        ret = ret.flat_name()
                    )
                }
            },
            Self::Object(v) => match v {
                ValueObject::Native(ref v) => format!("[Object 0x{:08x}]", *v as u32),
                _ => todo!(),
            },
            Self::Ref(var) => var.read().to_string_or_default(),
        }
    }

    pub fn as_object(&self) -> Option<&ValueObject> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }

    // Parsing related //

    pub fn compile_string(mut buffer: String, string: impl AsRef<str>) -> CompileResult {
        use std::fmt::Write;

        let string = string.as_ref();
        let len = string.len() as u8 as char;
        _ = buffer.write_char(len);
        _ = buffer.write_str(string);

        Ok(buffer)
    }

    pub fn visit_string(parser: Parser<'_>) -> ParserResult<'_, &str> {
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

    pub fn compile_slice<T>(mut buffer: String, slice: &[T]) -> CompileResult
    where
        T: Compilable,
    {
        use std::fmt::Write;

        let len = slice.len() as u8 as char;
        _ = buffer.write_char(len);

        for i in slice {
            buffer = i.compile_bytecode(buffer)?;
        }

        Ok(buffer)
    }

    pub fn visit_slice<T>(
        parser: Parser<'_>,
        inside_fn: impl Fn(Parser<'_>) -> ParserResult<'_, T>,
    ) -> ParserResult<'_, Vec<T>> {
        let (parser, len) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected fields length"))?;
        let len = len as u8;
        let mut slice = Vec::with_capacity(len as usize);

        let mut parser = parser;
        for _ in 0..len {
            let (parser_, val) = inside_fn(parser)?;
            parser = parser_;
            slice.push(val);
        }

        Ok((parser, slice))
    }
}

impl Compilable for Value {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        match self {
            Self::Null => {
                _ = buffer.write_char(VALUE_UNDEFINED);
            }
            Self::Bool(v) => {
                _ = buffer.write_char(VALUE_BOOL);
                _ = buffer.write_char(if *v { '\x01' } else { '\x00' });
            }
            Self::Char(v) => {
                _ = buffer.write_char(VALUE_CHAR);
                _ = buffer.write_char(*v);
            }

            Self::I16(v) => {
                _ = buffer.write_char(VALUE_I16);
                _ = buffer.write_str(&String::from_utf8_lossy(&[
                    if v.is_positive() { 1 } else { 0 },
                    (v.unsigned_abs() >> 8) as u8,
                    v.unsigned_abs() as u8,
                ]));
            }
            Self::U8(v) => {
                _ = buffer.write_char(VALUE_U8);
                _ = buffer.write_char(*v as char);
            }
            Self::F32(v) => {
                _ = buffer.write_char(VALUE_F32);
                _ = buffer.write_str(&v.to_string());
                _ = buffer.write_char(COMMAND_SEPARATOR);
            }

            Self::Fun(_) => todo!(),
            Self::Object(_) => todo!(),
            Self::Ref(_) => unimplemented!("Reference cannot be compiled"),
            Self::String(string) => {
                _ = buffer.write_char(VALUE_STRING);
                buffer = string.compile_bytecode(buffer)?;
            }
        }

        Ok(buffer)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Char(v) => write!(f, "{v:?}"),
            Self::Bool(v) => write!(f, "{v:?}"),

            Self::U8(v) => write!(f, "{v}u8"),
            Self::I16(v) => write!(f, "{v}i16"),
            Self::F32(v) => write!(f, "{v}f32"),

            Self::Fun(_) => f.write_str("[Function]"),
            Self::Object(_) => f.write_str("[Native Object]"),
            Self::Ref(var) => write!(f, "&{}", var.read()),
            Self::String(v) => write!(f, "{v:?}"),
        }
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Value::String(val.into())
    }
}

impl From<Value> for CommandExpression {
    fn from(val: Value) -> Self {
        CommandExpression::Value(val)
    }
}

impl From<Value> for Option<CommandExpression> {
    fn from(val: Value) -> Self {
        Some(CommandExpression::Value(val))
    }
}

impl From<Value> for Box<CommandExpression> {
    fn from(val: Value) -> Self {
        CommandExpression::Value(val).into()
    }
}

impl From<Value> for Option<Box<CommandExpression>> {
    fn from(val: Value) -> Self {
        CommandExpression::Value(val).into()
    }
}

impl Value {
    pub fn visit_u8(parser: Parser<'_>) -> ParserResult<'_, u8> {
        let (parser, value) = parser::anychar(parser)?;

        Ok((parser, value as u8))
    }

    pub fn visit_u16(parser: Parser<'_>) -> ParserResult<'_, u16> {
        let (parser, b1) = parser::anychar(parser)?;
        let (parser, b2) = parser::anychar(parser)?;

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

                let (parser, value) = parser::anychar(parser)
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

                let (parser, sign) = parser::anychar(parser)?;
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
                let mut parser = parser;
                loop {
                    let (_parser, b) = parser::anychar(parser)?;
                    parser = _parser;
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

    pub fn as_function(&self) -> Option<&ValueFun> {
        if let Self::Fun(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
