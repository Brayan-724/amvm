use crate::error::error_msgs;
use crate::{Compilable, Parser, ParserError, Value};

pub static EXPR_VALUE: char = '\x11';
pub static EXPR_VAR: char = '\x12';
pub static EXPR_PROP: char = '\x13';
pub static EXPR_ADD: char = '\x14';
pub static EXPR_SUB: char = '\x15';
pub static EXPR_COND: char = '\x16';

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
    Value(Value),
    Var(Value),

    Property(Box<CommandExpression>, Box<CommandExpression>),

    Addition(Box<CommandExpression>, Box<CommandExpression>),
    Substraction(Box<CommandExpression>, Box<CommandExpression>),

    BinaryCondition(
        BinaryConditionKind,
        Box<CommandExpression>,
        Box<CommandExpression>,
    ),
}

impl std::fmt::Display for CommandExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => (v as &dyn std::fmt::Display).fmt(f),
            Self::Var(v) => f.write_fmt(format_args!("Var({v})")),
            Self::Property(a, b) => f.write_fmt(format_args!("({a})[{b}]")),

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
    pub fn visit_cond(parser: &mut Parser) -> Result<Self, ParserError> {
        let kind = parser
            .consume()
            .ok_or_else(|| parser.error_corrupt(error_msgs::ERROR_INVALID_VALUE_DECL, "", 1))?;

        let mut condition = |kind: BinaryConditionKind| -> Result<Self, ParserError> {
            Ok(CommandExpression::BinaryCondition(
                kind,
                CommandExpression::visit(parser)?.into(),
                CommandExpression::visit(parser)?.into(),
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

            _ => Err(parser.error_corrupt(
                error_msgs::ERROR_UNKNOWN_VALUE_KIND,
                format!("{kind:?}"),
                1,
            )),
        }
    }

    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        let b = parser.consume().ok_or_else(|| {
            parser.error_corrupt(
                error_msgs::ERROR_INVALID_HEADER_DECL,
                "Can't get type casting kind",
                1,
            )
        })?;

        match b {
            b if b == EXPR_VAR => Ok(CommandExpression::Var(Value::visit(parser)?)),
            b if b == EXPR_ADD => Ok(CommandExpression::Addition(
                CommandExpression::visit(parser)?.into(),
                CommandExpression::visit(parser)?.into(),
            )),
            b if b == EXPR_SUB => Ok(CommandExpression::Substraction(
                CommandExpression::visit(parser)?.into(),
                CommandExpression::visit(parser)?.into(),
            )),
            b if b == EXPR_PROP => Ok(CommandExpression::Property(
                CommandExpression::visit(parser)?.into(),
                CommandExpression::visit(parser)?.into(),
            )),
            b if b == EXPR_VALUE => Ok(CommandExpression::Value(Value::visit(parser)?)),
            b if b == EXPR_COND => Self::visit_cond(parser),
            _ => Err(parser.error_corrupt("Unknown expression kind.", format!("{b:?}"), 1)),
        }
    }
}

impl Compilable for CommandExpression {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::Value(v) => Box::from(format!("{EXPR_VALUE}{}", v.compile_bytecode())),
            Self::Var(var) => Box::from(format!("{EXPR_VAR}{}", var.compile_bytecode())),
            Self::BinaryCondition(kind, a, b) => {
                let kind = kind.compile_bytecode();
                let a = a.compile_bytecode();
                let b = b.compile_bytecode();

                Box::from(format!("{EXPR_COND}{kind}{a}{b}"))
            }
            Self::Property(a, b) => Box::from(format!(
                "{EXPR_PROP}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            )),
            Self::Addition(a, b) => Box::from(format!(
                "{EXPR_ADD}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            )),
            Self::Substraction(a, b) => Box::from(format!(
                "{EXPR_SUB}{}{}",
                a.compile_bytecode(),
                b.compile_bytecode()
            )),
        })
    }
}
