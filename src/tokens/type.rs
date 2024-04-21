use std::fmt;

use crate::{create_bytes, parser, Compilable, Parser, ParserResult, Value};

create_bytes! {0;
    TYPE_ANON,
    TYPE_UNION,
    TYPE_TUPLE,
    TYPE_STRING,
    TYPE_U8,
    TYPE_CUSTOM
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmvmTypeDefinition {
    Inheritance(Vec<AmvmType>, Box<AmvmTypeDefinition>),
    Struct(Vec<(Box<str>, AmvmType)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmvmType {
    Anonymous,

    Union(Box<AmvmType>, Box<AmvmType>),
    Tuple(Vec<AmvmType>),

    Primitive(&'static str),
    Named(Box<str>),
}

impl AmvmType {
    pub fn flat_name(&self) -> String {
        match self {
            Self::Anonymous => String::from("#"),
            Self::Union(a, b) => format!("{}{}", a.flat_name(), b.flat_name()),
            Self::Tuple(tuple) => format!(
                "#({})",
                tuple
                    .iter()
                    .map(|v| v.flat_name())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Primitive(name) => format!("#{name}"),
            Self::Named(name) => format!("#{name}"),
        }
    }
}

impl fmt::Display for AmvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anonymous => f.write_str("Anonymous"),
            Self::Union(a, b) => f.write_fmt(format_args!("{a} + {b}")),
            Self::Tuple(types) => {
                let mut tuple = f.debug_tuple("Tuple");
                for r#type in types {
                    tuple.field(r#type);
                }

                tuple.finish()
            }
            Self::Named(name) => f.write_str(name),
            Self::Primitive(name) => f.write_str(name),
            // Self::Struct(_) => todo!(),
        }
    }
}

impl Compilable for AmvmType {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Anonymous => TYPE_ANON.to_string(),
            Self::Union(a, b) => format!(
                "{TYPE_UNION}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            ),
            Self::Tuple(fields) => {
                let len = fields.len() as u8 as char;
                let fields: String = fields.iter().map(|f| f.compile_bytecode()).collect();

                format!("{TYPE_TUPLE}{len}{fields}")
            }
            Self::Primitive("string") => TYPE_STRING.to_string(),
            Self::Primitive("u8") => TYPE_U8.to_string(),
            Self::Primitive(_) => unreachable!(),
            Self::Named(name) => format!("{TYPE_CUSTOM}{}", Value::compile_string(name)),
        })
    }
}

impl AmvmType {
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, AmvmType> {
        let (parser, c) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected type kind"))?;

        Ok(match c {
            _ if c == TYPE_ANON => (parser, AmvmType::Anonymous),
            _ if c == TYPE_UNION => {
                let (parser, a) = AmvmType::visit(parser)?;
                let (parser, b) = AmvmType::visit(parser)?;
                (parser, AmvmType::Union(Box::new(a), Box::new(b)))
            }
            _ if c == TYPE_TUPLE => {
                let (parser, fields_len) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected fields length"))?;
                let fields_len = fields_len as u8;
                let mut fields = Vec::with_capacity(fields_len as usize);

                let mut parser = parser;
                for _ in 0..fields_len {
                    let (parser_, r#type) = AmvmType::visit(parser)?;
                    parser = parser_;
                    fields.push(r#type);
                }
                (parser, AmvmType::Tuple(fields))
            }
            _ if c == TYPE_STRING => (parser, AmvmType::Primitive("string")),
            _ if c == TYPE_U8 => (parser, AmvmType::Primitive("u8")),
            _ if c == TYPE_ANON => (parser, AmvmType::Anonymous),
            _ if c == TYPE_CUSTOM => Value::visit_string(parser)
                .map(|(parser, type_name)| (parser, AmvmType::Named(Box::from(type_name))))?,
            _ => unimplemented!("{:02x}", c as u8),
        })
    }
}

impl Compilable for Vec<(Box<str>, AmvmType)> {
    fn compile_bytecode(&self) -> Box<str> {
        let len = self.len() as u8 as char;

        let fields: String = self
            .iter()
            .map(|f| {
                format!(
                    "{name}{value}",
                    name = f.0.compile_bytecode(),
                    value = f.1.compile_bytecode()
                )
            })
            .collect();

        Box::from(format!("{len}{fields}"))
    }
}
