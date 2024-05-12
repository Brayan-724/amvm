use std::fmt;

use crate::CompileResult;
use crate::{
    create_bytes,
    parser::{self, Parser, ParserResult},
    tokens::Value,
    Compilable,
};

create_bytes! {0;
    TYPE_ANON,
    TYPE_CUSTOM,

    TYPE_TUPLE,
    TYPE_UNION,

    TYPE_BOOL,
    TYPE_FUN,
    TYPE_STRING,
    TYPE_U8
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmvmTypeDefinition {
    Inheritance(Vec<AmvmType>, Box<AmvmTypeDefinition>),
    Struct {
        generics: Vec<(Box<str>, Option<AmvmType>)>,
        fields: Vec<(Box<str>, AmvmType)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmvmPrimitiveType {
    U8,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmvmType {
    Anonymous,

    Tuple(Vec<AmvmType>),
    Union(Box<AmvmType>, Box<AmvmType>),

    Fun(Vec<AmvmType>, Box<AmvmType>),
    Named(Box<str>),
    Primitive(AmvmPrimitiveType),
}

impl AmvmType {
    pub fn flat_name_args(args: &Vec<AmvmType>) -> String {
        use std::fmt::Write;

        args.iter()
            .enumerate()
            .fold(String::new(), |mut buffer, (idx, a)| {
                _ = if idx == 0 {
                    write!(buffer, "{}", a.flat_name())
                } else {
                    write!(buffer, ", {}", a.flat_name())
                };

                buffer
            })
    }

    pub fn flat_name(&self) -> String {
        match self {
            Self::Anonymous => String::from("#"),
            Self::Named(name) => format!("#{name}"),

            Self::Tuple(tuple) => format!(
                "#({})",
                tuple
                    .iter()
                    .map(|v| v.flat_name())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Union(a, b) => format!("{}{}", a.flat_name(), b.flat_name()),

            Self::Fun(args, ret) => format!(
                "#Fun({}) -> {}",
                Self::flat_name_args(args),
                ret.flat_name()
            ),
            Self::Primitive(name) => format!("#{name}"),
        }
    }

    pub fn fmt_args(args: &Vec<AmvmType>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, a) in args.iter().enumerate() {
            if idx == 0 {
                write!(f, "{}", a.flat_name())?;
            } else {
                write!(f, ", {}", a.flat_name())?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for AmvmPrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => f.write_str("bool"),
            Self::U8 => f.write_str("u8"),
            Self::String => f.write_str("string"),
        }
    }
}

impl fmt::Display for AmvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anonymous => f.write_str("Anonymous"),
            Self::Named(name) => f.write_str(name),

            Self::Union(a, b) => write!(f, "{a} + {b}"),
            Self::Tuple(types) => {
                let mut tuple = f.debug_tuple("Tuple");
                for r#type in types {
                    tuple.field(r#type);
                }

                tuple.finish()
            }

            Self::Fun(args, ret) => {
                f.write_str("Fn(")?;
                AmvmType::fmt_args(args, f)?;
                write!(f, ") -> {ret}")
            }
            Self::Primitive(name) => name.fmt(f),
            // Self::Struct(_) => todo!(),
        }
    }
}

impl Compilable for AmvmType {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        match self {
            Self::Anonymous => _ = buffer.write_char(TYPE_ANON),
            Self::Named(name) => {
                _ = buffer.write_char(TYPE_CUSTOM);
                buffer = name.compile_bytecode(buffer)?;
            }

            Self::Union(a, b) => {
                _ = buffer.write_char(TYPE_UNION);
                buffer = a.compile_bytecode(buffer)?;
                buffer = b.compile_bytecode(buffer)?;
            }
            Self::Tuple(fields) => {
                _ = buffer.write_char(TYPE_TUPLE);
                buffer = Value::compile_slice(buffer, fields)?;
            }

            Self::Fun(args, ret) => {
                _ = buffer.write_char(TYPE_FUN);
                buffer = Value::compile_slice(buffer, args)?;
                buffer = ret.compile_bytecode(buffer)?;
            }

            Self::Primitive(ty) => {
                _ = buffer.write_char(match ty {
                    AmvmPrimitiveType::Bool => TYPE_BOOL,
                    AmvmPrimitiveType::String => TYPE_STRING,
                    AmvmPrimitiveType::U8 => TYPE_U8,
                })
            }
        }

        Ok(buffer)
    }
}

impl AmvmType {
    pub fn visit(parser: Parser<'_>) -> ParserResult<'_, AmvmType> {
        let (parser, c) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected type kind"))?;

        Ok(match c {
            _ if c == TYPE_ANON => (parser, AmvmType::Anonymous),
            _ if c == TYPE_CUSTOM => Value::visit_string(parser)
                .map(|(parser, type_name)| (parser, AmvmType::Named(Box::from(type_name))))?,

            _ if c == TYPE_UNION => {
                let (parser, a) = AmvmType::visit(parser)?;
                let (parser, b) = AmvmType::visit(parser)?;
                (parser, AmvmType::Union(Box::new(a), Box::new(b)))
            }
            _ if c == TYPE_TUPLE => {
                let (parser, fields) = Value::visit_slice(parser, AmvmType::visit)?;
                (parser, AmvmType::Tuple(fields))
            }

            _ if c == TYPE_FUN => {
                let (parser, args) = Value::visit_slice(parser, AmvmType::visit)?;
                let (parser, ret) = AmvmType::visit(parser)?;

                (parser, AmvmType::Fun(args, Box::new(ret)))
            }

            // Primitives
            _ if c == TYPE_BOOL => (parser, AmvmType::Primitive(AmvmPrimitiveType::Bool)),
            _ if c == TYPE_STRING => (parser, AmvmType::Primitive(AmvmPrimitiveType::String)),
            _ if c == TYPE_U8 => (parser, AmvmType::Primitive(AmvmPrimitiveType::U8)),

            _ => return Err(parser.error(parser::VerboseErrorKind::Char(c), true)),
        })
    }
}
