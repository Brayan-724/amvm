use crate::{
    create_bytes,
    parser::{self, Parser, ParserResult},
    tokens::Value,
    Compilable,
};

use super::AmvmType;

create_bytes! {0x10;
    EXPR_BINARY,
    EXPR_PREV,
    EXPR_PROP,
    EXPR_RANGE,
    EXPR_STRUCT,
    EXPR_VALUE,
    EXPR_VAR
}

create_bytes! {0x0;
    EXPR_KIND_ADD,
    EXPR_KIND_SUB,
    EXPR_KIND_MUL,

    /// Conditionals
    EXPR_KIND_EQUAL,
    EXPR_KIND_NOT_EQUAL,
    EXPR_KIND_GREATER_THAN,
    EXPR_KIND_GREATER_THAN_EQUAL,
    EXPR_KIND_LESS_THAN,
    EXPR_KIND_LESS_THAN_EQUAL
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryKind {
    Add,
    Sub,
    Mult,

    // Conditionals
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
}

impl Compilable for BinaryKind {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Add => EXPR_KIND_ADD.to_string(),
            Self::Sub => EXPR_KIND_SUB.to_string(),
            Self::Mult => EXPR_KIND_MUL.to_string(),
            // Conditionals
            Self::Equal => EXPR_KIND_EQUAL.to_string(),
            Self::NotEqual => EXPR_KIND_NOT_EQUAL.to_string(),
            Self::GreaterThan => EXPR_KIND_GREATER_THAN.to_string(),
            Self::GreaterThanEqual => EXPR_KIND_GREATER_THAN_EQUAL.to_string(),
            Self::LessThan => EXPR_KIND_LESS_THAN.to_string(),
            Self::LessThanEqual => EXPR_KIND_LESS_THAN_EQUAL.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandExpression {
    Binary(BinaryKind, Box<CommandExpression>, Box<CommandExpression>),
    Prev,
    Property(Box<CommandExpression>, Box<CommandExpression>),
    Range(Box<CommandExpression>, Box<CommandExpression>),
    Struct(AmvmType, Vec<(Box<str>, CommandExpression)>),
    Value(Value),
    Var(String),
}

impl std::fmt::Display for CommandExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(kind, a, b) => write!(f, "{a} {kind:?} {b}"),
            Self::Prev => f.write_str("Prev"),
            Self::Property(a, b) => write!(f, "({a})[{b}]"),
            Self::Range(a, b) => write!(f, "({a}) .. ({b})"),
            Self::Struct(t, data) => write!(f, "{t} {data:?}"),
            Self::Value(v) => (v as &dyn std::fmt::Display).fmt(f),
            Self::Var(v) => write!(f, "${v}"),
        }
    }
}

impl Into<Option<Box<CommandExpression>>> for CommandExpression {
    fn into(self) -> Option<Box<CommandExpression>> {
        Some(Box::from(self))
    }
}

impl CommandExpression {
    #[inline]
    fn condition<'a>(parser: Parser<'a>, kind: BinaryKind) -> ParserResult<'a, Self> {
        let (parser, a) = CommandExpression::visit(parser)?;
        let (parser, b) = CommandExpression::visit(parser)?;

        Ok((parser, CommandExpression::Binary(kind, a.into(), b.into())))
    }

    pub fn visit_binary<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, kind) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected conditional kind"))?;

        match kind {
            k if k == EXPR_KIND_ADD => Self::condition(parser, BinaryKind::Add),
            k if k == EXPR_KIND_SUB => Self::condition(parser, BinaryKind::Sub),
            k if k == EXPR_KIND_MUL => Self::condition(parser, BinaryKind::Mult),

            k if k == EXPR_KIND_EQUAL => Self::condition(parser, BinaryKind::Equal),
            k if k == EXPR_KIND_NOT_EQUAL => Self::condition(parser, BinaryKind::NotEqual),

            k if k == EXPR_KIND_GREATER_THAN => Self::condition(parser, BinaryKind::GreaterThan),
            k if k == EXPR_KIND_GREATER_THAN_EQUAL => {
                Self::condition(parser, BinaryKind::GreaterThanEqual)
            }

            k if k == EXPR_KIND_LESS_THAN => Self::condition(parser, BinaryKind::LessThan),
            k if k == EXPR_KIND_LESS_THAN_EQUAL => {
                Self::condition(parser, BinaryKind::LessThanEqual)
            }

            _ => Err(parser.error(
                parser::VerboseErrorKind::Context("Unknown conditional kind"),
                true,
            )),
        }
    }

    pub fn visit<'a>(parser_: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, b) = parser::anychar(parser_)
            .map_err(parser_.nom_err_with_context("Expected expression kind"))?;

        match b {
            _ if b == EXPR_BINARY => Self::visit_binary(parser),
            _ if b == EXPR_PREV => Ok((parser, CommandExpression::Prev)),
            _ if b == EXPR_PROP => {
                let (parser, a) = CommandExpression::visit(parser)?;
                let (parser, b) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Property(a.into(), b.into())))
            }
            _ if b == EXPR_RANGE => {
                let (parser, a) = CommandExpression::visit(parser)?;
                let (parser, b) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Range(a.into(), b.into())))
            }
            _ if b == EXPR_STRUCT => {
                let (parser, r#type) = AmvmType::visit(parser)?;
                let (parser, fields_len) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected fields length"))?;

                let fields_len = fields_len as u8;
                let mut data = Vec::with_capacity(fields_len as usize);

                let mut parser = parser;
                for _ in 0..fields_len {
                    let (parser_, field) = Value::visit_string(parser)?;
                    let (parser_, r#type) = CommandExpression::visit(parser_)?;
                    parser = parser_;
                    data.push((Box::from(field), r#type))
                }

                Ok((parser, CommandExpression::Struct(r#type, data)))
            }
            _ if b == EXPR_VALUE => {
                let (parser, value) = Value::visit(parser)?;

                Ok((parser, CommandExpression::Value(value)))
            }
            _ if b == EXPR_VAR => {
                let (parser, value) = Value::visit_string(parser)?;
                let value = value.to_owned();
                Ok((parser, CommandExpression::Var(value)))
            }
            _ => Err(parser_.error(
                parser::VerboseErrorKind::Context("Unknown expression kind"),
                true,
            )),
        }
    }
}

impl Compilable for CommandExpression {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Binary(kind, a, b) => {
                let kind = kind.compile_bytecode();
                let a = a.compile_bytecode();
                let b = b.compile_bytecode();

                format!("{EXPR_BINARY}{kind}{a}{b}")
            }
            Self::Prev => EXPR_PREV.to_string(),
            Self::Property(a, b) => format!(
                "{EXPR_PROP}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            ),
            Self::Range(a, b) => format!(
                "{EXPR_RANGE}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            ),
            Self::Struct(r#type, data) => format!(
                "{EXPR_STRUCT}{}{}",
                r#type.compile_bytecode(),
                data.compile_bytecode()
            ),
            Self::Value(v) => format!("{EXPR_VALUE}{}", v.compile_bytecode()),
            Self::Var(var) => format!("{EXPR_VAR}{}", Value::compile_string(var)),
        })
    }
}

impl Compilable for Vec<(Box<str>, CommandExpression)> {
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
