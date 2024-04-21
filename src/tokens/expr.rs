use crate::parser::ParserResult;
use crate::{parser, Compilable, Parser, Value};

use super::AmvmType;

pub static EXPR_VALUE: char = '\x11';
pub static EXPR_VAR: char = '\x12';
pub static EXPR_PROP: char = '\x13';
pub static EXPR_ADD: char = '\x14';
pub static EXPR_SUB: char = '\x15';
pub static EXPR_COND: char = '\x16';
pub static EXPR_PREV: char = '\x18';
pub static EXPR_STRUCT: char = '\x19';

pub static EXPR_COND_LESS_THAN: char = '\x01';
pub static EXPR_COND_LESS_THAN_EQUAL: char = '\x02';
pub static EXPR_COND_GREATER_THAN: char = '\x03';
pub static EXPR_COND_GREATER_THAN_EQUAL: char = '\x04';
pub static EXPR_COND_EQUAL: char = '\x05';
pub static EXPR_COND_NOT_EQUAL: char = '\x06';

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryConditionKind {
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Equal,
    NotEqual,
}

impl Compilable for BinaryConditionKind {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::LessThan => EXPR_COND_LESS_THAN.to_string(),
            Self::LessThanEqual => EXPR_COND_LESS_THAN_EQUAL.to_string(),
            Self::GreaterThan => EXPR_COND_GREATER_THAN.to_string(),
            Self::GreaterThanEqual => EXPR_COND_GREATER_THAN_EQUAL.to_string(),
            Self::Equal => EXPR_COND_EQUAL.to_string(),
            Self::NotEqual => EXPR_COND_NOT_EQUAL.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandExpression {
    Prev,

    Value(Value),
    Var(String),

    Property(Box<CommandExpression>, Box<CommandExpression>),

    Addition(Box<CommandExpression>, Box<CommandExpression>),
    Substraction(Box<CommandExpression>, Box<CommandExpression>),

    BinaryCondition(
        BinaryConditionKind,
        Box<CommandExpression>,
        Box<CommandExpression>,
    ),

    Struct(AmvmType, Vec<(Box<str>, CommandExpression)>),
}

impl std::fmt::Display for CommandExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prev => f.write_str("Prev"),
            Self::Value(v) => (v as &dyn std::fmt::Display).fmt(f),
            Self::Var(v) => f.write_fmt(format_args!("${v}")),
            Self::Property(a, b) => f.write_fmt(format_args!("({a})[{b}]")),
            Self::Struct(t, data) => f.write_fmt(format_args!("{t} {data:?}")),

            Self::Addition(a, b) => f.write_fmt(format_args!("{a} + {b}")),
            Self::Substraction(a, b) => f.write_fmt(format_args!("{a} - {b}")),
            Self::BinaryCondition(kind, a, b) => f.write_fmt(format_args!("{a} {kind:?} {b}")),
        }
    }
}

impl Into<Option<Box<CommandExpression>>> for CommandExpression {
    fn into(self) -> Option<Box<CommandExpression>> {
        Some(Box::from(self))
    }
}

impl CommandExpression {
    pub fn visit_cond<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, kind) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected conditional kind"))?;

        let condition = |kind: BinaryConditionKind| -> ParserResult<'a, Self> {
            let (parser, a) = CommandExpression::visit(parser)?;
            let (parser, b) = CommandExpression::visit(parser)?;

            Ok((
                parser,
                CommandExpression::BinaryCondition(kind, a.into(), b.into()),
            ))
        };

        match kind {
            k if k == EXPR_COND_LESS_THAN => condition(BinaryConditionKind::LessThan),
            k if k == EXPR_COND_LESS_THAN_EQUAL => condition(BinaryConditionKind::LessThanEqual),

            k if k == EXPR_COND_GREATER_THAN => condition(BinaryConditionKind::GreaterThan),
            k if k == EXPR_COND_GREATER_THAN_EQUAL => {
                condition(BinaryConditionKind::GreaterThanEqual)
            }

            k if k == EXPR_COND_EQUAL => condition(BinaryConditionKind::Equal),
            k if k == EXPR_COND_NOT_EQUAL => condition(BinaryConditionKind::NotEqual),

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
            b if b == EXPR_PREV => Ok((parser, CommandExpression::Prev)),
            b if b == EXPR_VAR => {
                let (parser, value) = Value::visit_string(parser)?;
                let value = value.to_owned();
                Ok((parser, CommandExpression::Var(value)))
            }
            b if b == EXPR_ADD => {
                let (parser, a) = CommandExpression::visit(parser)?;
                let (parser, b) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Addition(a.into(), b.into())))
            }
            b if b == EXPR_SUB => {
                let (parser, a) = CommandExpression::visit(parser)?;
                let (parser, b) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Substraction(a.into(), b.into())))
            }
            b if b == EXPR_PROP => {
                let (parser, a) = CommandExpression::visit(parser)?;
                let (parser, b) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Property(a.into(), b.into())))
            }
            b if b == EXPR_VALUE => {
                let (parser, value) = Value::visit(parser)?;

                Ok((parser, CommandExpression::Value(value)))
            }
            b if b == EXPR_COND => Self::visit_cond(parser),
            b if b == EXPR_STRUCT => {
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
            Self::Prev => EXPR_PREV.to_string(),
            Self::Value(v) => format!("{EXPR_VALUE}{}", v.compile_bytecode()),
            Self::Var(var) => format!("{EXPR_VAR}{}", Value::compile_string(var)),
            Self::Struct(r#type, data) => format!(
                "{EXPR_STRUCT}{}{}",
                r#type.compile_bytecode(),
                data.compile_bytecode()
            ),
            Self::BinaryCondition(kind, a, b) => {
                let kind = kind.compile_bytecode();
                let a = a.compile_bytecode();
                let b = b.compile_bytecode();

                format!("{EXPR_COND}{kind}{a}{b}")
            }
            Self::Property(a, b) => format!(
                "{EXPR_PROP}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            ),
            Self::Addition(a, b) => {
                format!("{EXPR_ADD}{}{}", a.compile_bytecode(), b.compile_bytecode())
            }
            Self::Substraction(a, b) => {
                format!("{EXPR_SUB}{}{}", a.compile_bytecode(), b.compile_bytecode())
            }
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
