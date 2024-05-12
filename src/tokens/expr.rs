use crate::{
    create_bytes,
    parser::{self, Parser, ParserResult},
    tokens::{AmvmType, Command, Value, VariableKind},
    Compilable, CompileResult,
};

create_bytes! {0x10;
    EXPR_BINARY,
    EXPR_PREV,
    EXPR_PROP,
    EXPR_RANGE,
    EXPR_REF,
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
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        _ = buffer.write_char(match self {
            Self::Add => EXPR_KIND_ADD,
            Self::Sub => EXPR_KIND_SUB,
            Self::Mult => EXPR_KIND_MUL,
            // Conditionals
            Self::Equal => EXPR_KIND_EQUAL,
            Self::NotEqual => EXPR_KIND_NOT_EQUAL,
            Self::GreaterThan => EXPR_KIND_GREATER_THAN,
            Self::GreaterThanEqual => EXPR_KIND_GREATER_THAN_EQUAL,
            Self::LessThan => EXPR_KIND_LESS_THAN,
            Self::LessThanEqual => EXPR_KIND_LESS_THAN_EQUAL,
        });

        Ok(buffer)
    }
}

#[derive(Debug, Clone)]
pub enum CommandExpression {
    Binary(BinaryKind, Box<CommandExpression>, Box<CommandExpression>),
    Prev,
    Property(Box<CommandExpression>, Box<CommandExpression>),
    Range(Box<CommandExpression>, Box<CommandExpression>),
    Ref(VariableKind, Box<CommandExpression>),
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
            Self::Ref(kind, var) => write!(f, "&{kind} {var}"),
            Self::Struct(t, data) => write!(f, "{t} {data:?}"),
            Self::Value(v) => (v as &dyn std::fmt::Display).fmt(f),
            Self::Var(v) => write!(f, "${v}"),
        }
    }
}

impl From<CommandExpression> for Option<Box<CommandExpression>> {
    fn from(val: CommandExpression) -> Self {
        Some(Box::from(val))
    }
}

impl CommandExpression {
    #[inline]
    fn condition(parser: Parser<'_>, kind: BinaryKind) -> ParserResult<'_, Self> {
        let (parser, a) = CommandExpression::visit(parser)?;
        let (parser, b) = CommandExpression::visit(parser)?;

        Ok((parser, CommandExpression::Binary(kind, a.into(), b.into())))
    }

    pub fn visit_binary(parser: Parser<'_>) -> ParserResult<'_, Self> {
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

    pub fn visit(parser_: Parser<'_>) -> ParserResult<'_, Self> {
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
            _ if b == EXPR_REF => {
                let (parser, kind) = Command::visit_kind(parser)?;
                let (parser, var) = CommandExpression::visit(parser)?;

                Ok((parser, CommandExpression::Ref(kind, var.into())))
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
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        use std::fmt::Write;

        match self {
            Self::Binary(kind, a, b) => {
                _ = buffer.write_char(EXPR_BINARY);
                buffer = kind.compile_bytecode(buffer)?;
                buffer = a.compile_bytecode(buffer)?;
                buffer = b.compile_bytecode(buffer)?;
            }
            Self::Prev => _ = buffer.write_char(EXPR_PREV),
            Self::Property(a, b) => {
                _ = buffer.write_char(EXPR_PROP);
                buffer = a.compile_bytecode(buffer)?;
                buffer = b.compile_bytecode(buffer)?;
            }
            Self::Range(a, b) => {
                _ = buffer.write_char(EXPR_RANGE);
                buffer = a.compile_bytecode(buffer)?;
                buffer = b.compile_bytecode(buffer)?;
            }
            Self::Ref(kind, var) => {
                _ = buffer.write_char(EXPR_REF);
                buffer = kind.compile_bytecode(buffer)?;
                buffer = var.compile_bytecode(buffer)?;
            }
            Self::Struct(r#type, data) => {
                _ = buffer.write_char(EXPR_STRUCT);
                buffer = r#type.compile_bytecode(buffer)?;
                buffer = (
                    data,
                    |mut buffer: String, f: &(Box<str>, CommandExpression)| -> CompileResult {
                        buffer = f.0.compile_bytecode(buffer)?;
                        buffer = f.1.compile_bytecode(buffer)?;
                        Ok(buffer)
                    },
                )
                    .compile_bytecode(buffer)?;
            }
            Self::Value(v) => {
                _ = buffer.write_char(EXPR_VALUE);
                buffer = v.compile_bytecode(buffer)?;
            }
            Self::Var(var) => {
                _ = buffer.write_char(EXPR_VAR);
                buffer = var.compile_bytecode(buffer)?;
            }
        }

        Ok(buffer)
    }
}
