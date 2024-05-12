use std::fmt::{self, Write};

use crate::CompileResult;
use crate::{
    create_bytes,
    parser::{self, Parser, ParserResult},
    tokens::{
        AmvmType, CommandExpression, Value, VariableKind, COMMAND_SEPARATOR, VAR_CONST, VAR_LET,
        VAR_MUT, VAR_VAR,
    },
    Compilable,
};

create_bytes! {0x50;
    CMD_ASGN_VAR,
    CMD_BREAK,
    CMD_BUILTIN,
    CMD_CALL,
    CMD_COND,
    CMD_DCLR_VAR,
    CMD_FN,
    CMD_FOR,
    CMD_META,
    CMD_META_FILE,
    CMD_LOOP,
    CMD_PUSH,
    CMD_PUTS,
    CMD_RET,
    CMD_SCOPE,
    CMD_STRUCT
}

#[derive(Debug, Clone)]
pub enum Command {
    AssignVariable {
        name: Box<str>,
        value: CommandExpression,
    },

    Break,

    Builtin {
        name: Box<str>,
        args: Vec<CommandExpression>,
    },

    Call {
        name: CommandExpression,
        args: Vec<CommandExpression>,
    },

    Conditional {
        condition: CommandExpression,
        body: Vec<Command>,
        otherwise: Option<Vec<Command>>,
    },

    DeclareVariable {
        name: Box<str>,
        kind: VariableKind,
        value: CommandExpression,
    },

    For {
        var: Box<str>,
        iterator: CommandExpression,
        body: Vec<Command>,
    },

    Function {
        name: Box<str>,
        args: Vec<(Box<str>, VariableKind, AmvmType)>,
        ret: AmvmType,
        body: Vec<Command>,
    },

    Meta {
        pos: (u16, u16),
        code: Box<str>,
    },

    MetaFile(Box<str>),

    Loop {
        body: Vec<Command>,
    },

    Push {
        value: CommandExpression,
    },

    Puts {
        value: CommandExpression,
    },

    Return {
        value: CommandExpression,
    },

    Scope {
        body: Vec<Command>,
    },

    Struct {
        name: Box<str>,
        body: Vec<(Box<str>, AmvmType)>,
    },
}

impl Command {
    pub fn visit_asgn(parser: Parser<'_>) -> ParserResult<'_, Self> {
        let (parser, name) = Value::visit_string(parser)?;
        let (parser, value) = CommandExpression::visit(parser)?;

        Ok((
            parser,
            Command::AssignVariable {
                name: name.into(),
                value,
            },
        ))
    }

    pub fn visit_kind(parser: Parser<'_>) -> ParserResult<'_, VariableKind> {
        let (parser, kind) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected variable kind"))?;

        let kind = match kind {
            b if b == VAR_CONST => VariableKind::Const,
            b if b == VAR_MUT => VariableKind::Mut,
            b if b == VAR_LET => VariableKind::Let,
            b if b == VAR_VAR => VariableKind::Var,
            _ => {
                return Err(parser.error(
                    parser::VerboseErrorKind::Context("Unknown variable kind"),
                    true,
                ))
            }
        };

        Ok((parser, kind))
    }

    pub fn visit_var(parser: Parser<'_>) -> ParserResult<'_, Self> {
        let (parser_, kind) = parser::anychar(parser)
            .map_err(parser.nom_err_with_context("Expected variable kind"))?;

        let kind = match kind {
            b if b == VAR_CONST => VariableKind::Const,
            b if b == VAR_MUT => VariableKind::Mut,
            b if b == VAR_LET => VariableKind::Let,
            b if b == VAR_VAR => VariableKind::Var,
            _ => {
                return Err(parser.error(
                    parser::VerboseErrorKind::Context("Unknown variable kind"),
                    true,
                ))
            }
        };
        let parser = parser_;

        let (parser, name) = Value::visit_string(parser)?;
        let (parser, value) = CommandExpression::visit(parser)?;

        Ok((
            parser,
            Command::DeclareVariable {
                name: name.into(),
                kind,
                value,
            },
        ))
    }

    fn visit_scope(parser: Parser<'_>) -> ParserResult<'_, Vec<Self>> {
        Value::visit_slice(parser, Command::visit)
    }

    pub fn visit(parser_: Parser<'_>) -> ParserResult<'_, Self> {
        let (parser, b) = parser::anychar(parser_)
            .map_err(parser_.nom_err_with_context("Expected command kind"))?;

        let (parser, value) = match b {
            _ if b == CMD_ASGN_VAR => Self::visit_asgn(parser)?,

            _ if b == CMD_BREAK => (parser, Command::Break),

            _ if b == CMD_BUILTIN => {
                let (parser, name) = Value::visit_string(parser)?;
                let (parser, args) = Value::visit_slice(parser, CommandExpression::visit)?;

                (
                    parser,
                    Command::Builtin {
                        name: name.into(),
                        args,
                    },
                )
            }

            _ if b == CMD_CALL => {
                let (parser, name) = CommandExpression::visit(parser)?;
                let (parser, args) = Value::visit_slice(parser, CommandExpression::visit)?;

                (parser, Command::Call { name, args })
            }

            _ if b == CMD_COND => {
                let (parser, condition) = CommandExpression::visit(parser)?;
                let (parser, body) = Self::visit_scope(parser)?;
                let (parser, otherwise) = if parser.peek(0) == Some(COMMAND_SEPARATOR) {
                    let (_, parser) = parser::take(1usize)(parser)?;
                    (parser, None)
                } else {
                    let (parser, otherwise) = Self::visit_scope(parser)?;
                    (parser, Some(otherwise))
                };

                (
                    parser,
                    Command::Conditional {
                        condition,
                        body,
                        otherwise,
                    },
                )
            }

            _ if b == CMD_DCLR_VAR => Self::visit_var(parser)?,

            _ if b == CMD_FN => {
                let (parser, name) = Value::visit_string(parser)?;
                let (parser, args) = Value::visit_slice(parser, |parser| {
                    let (parser, name) = Value::visit_string(parser)?;
                    let (parser, kind) = Self::visit_kind(parser)?;
                    let (parser, ty) = AmvmType::visit(parser)?;

                    Ok((parser, (Box::from(name), kind, ty)))
                })?;
                let (parser, ret) = AmvmType::visit(parser)?;
                let (parser, body) = Self::visit_scope(parser)?;

                (
                    parser,
                    Command::Function {
                        name: name.into(),
                        args,
                        ret,
                        body,
                    },
                )
            }

            _ if b == CMD_FOR => {
                let (parser, var) = Value::visit_string(parser)?;
                let (parser, iterator) = CommandExpression::visit(parser)?;
                let (parser, body) = Self::visit_scope(parser)?;

                (
                    parser,
                    Command::For {
                        var: var.into(),
                        iterator,
                        body,
                    },
                )
            }

            _ if b == CMD_LOOP => {
                let (parser, body) = Self::visit_scope(parser)?;
                (parser, Command::Loop { body })
            }

            _ if b == CMD_META => {
                let (parser, line) = Value::visit_u16(parser)?;
                let (parser, col) = Value::visit_u16(parser)?;
                let (parser, code) = Value::visit_string(parser)?;

                (
                    parser,
                    Command::Meta {
                        pos: (line, col),
                        code: code.into(),
                    },
                )
            }

            _ if b == CMD_META_FILE => {
                let (parser, file_name) = Value::visit_string(parser)?;

                (parser, Command::MetaFile(file_name.into()))
            }

            _ if b == CMD_PUSH => {
                let span = tracing::trace_span!("CMD_PUSH");
                let _span = span.enter();

                let (parser, value) = CommandExpression::visit(parser)?;
                (parser, Command::Push { value })
            }

            _ if b == CMD_PUTS => {
                let span = tracing::trace_span!("CMD_PUTS");
                let _span = span.enter();

                let (parser, value) = CommandExpression::visit(parser)?;
                (parser, Command::Puts { value })
            }

            _ if b == CMD_RET => {
                let (parser, value) = CommandExpression::visit(parser)?;
                (parser, Command::Return { value })
            }

            _ if b == CMD_SCOPE => {
                let (parser, body) = Self::visit_scope(parser)?;
                (parser, Command::Scope { body })
            }

            _ if b == CMD_STRUCT => {
                let (parser, name) = Value::visit_string(parser)?;
                let (parser, body) = Value::visit_slice(parser, |parser| {
                    let (parser, name) = Value::visit_string(parser)?;
                    let (parser, ty) = AmvmType::visit(parser)?;

                    Ok((parser, (Box::from(name), ty)))
                })?;

                let name = Box::from(name);
                (parser, Command::Struct { name, body })
            }

            _ => {
                return Err(parser_.error(
                    parser::VerboseErrorKind::Context("Unknown command kind"),
                    true,
                ))
            }
        };

        Ok((parser, value))
    }
}
impl Compilable for Command {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        match self {
            Self::AssignVariable { name, value } => {
                _ = buffer.write_char(CMD_ASGN_VAR);
                buffer = name.compile_bytecode(buffer)?;
                buffer = value.compile_bytecode(buffer)?;
            }
            Self::Break => _ = buffer.write_char(CMD_BREAK),
            Self::Builtin { name, args } => {
                _ = buffer.write_char(CMD_BUILTIN);
                buffer = name.compile_bytecode(buffer)?;
                buffer = Value::compile_slice(buffer, args)?;
            }
            Self::Call { name, args } => {
                _ = buffer.write_char(CMD_CALL);
                buffer = name.compile_bytecode(buffer)?;
                buffer = Value::compile_slice(buffer, args)?;
            }
            Self::Conditional {
                condition,
                body,
                otherwise,
            } => {
                _ = buffer.write_char(CMD_COND);
                buffer = condition.compile_bytecode(buffer)?;
                buffer = body.compile_bytecode(buffer)?;
                if let Some(otherwise) = otherwise {
                    _ = buffer = otherwise.compile_bytecode(buffer)?;
                } else {
                    _ = buffer.write_char(COMMAND_SEPARATOR);
                }
            }
            Self::DeclareVariable { name, value, kind } => {
                _ = buffer.write_char(CMD_DCLR_VAR);
                buffer = kind.compile_bytecode(buffer)?;
                buffer = name.compile_bytecode(buffer)?;
                buffer = value.compile_bytecode(buffer)?;
            }
            Self::For {
                var,
                iterator,
                body,
            } => {
                _ = buffer.write_char(CMD_FOR);
                buffer = var.compile_bytecode(buffer)?;
                buffer = iterator.compile_bytecode(buffer)?;
                buffer = body.compile_bytecode(buffer)?;
            }
            Self::Function {
                name,
                args,
                ret,
                body,
            } => {
                _ = buffer.write_char(CMD_FN);
                buffer = name.compile_bytecode(buffer)?;
                buffer = (
                    args,
                    |mut buffer: String,
                     arg: &(Box<str>, VariableKind, AmvmType)|
                     -> CompileResult {
                        buffer = arg.0.compile_bytecode(buffer)?;
                        buffer = arg.1.compile_bytecode(buffer)?;
                        buffer = arg.2.compile_bytecode(buffer)?;
                        Ok(buffer)
                    },
                )
                    .compile_bytecode(buffer)?;
                buffer = ret.compile_bytecode(buffer)?;
                buffer = body.compile_bytecode(buffer)?;
            }
            Self::Loop { body } => {
                _ = buffer.write_char(CMD_LOOP);
                buffer = body.compile_bytecode(buffer)?;
            }
            Self::Meta { pos, code } => {
                _ = buffer.write_char(CMD_META);
                buffer = pos.0.compile_bytecode(buffer)?;
                buffer = pos.1.compile_bytecode(buffer)?;
                buffer = code.compile_bytecode(buffer)?;
            }
            Self::MetaFile(file_name) => {
                _ = buffer.write_char(CMD_META_FILE);
                buffer = file_name.compile_bytecode(buffer)?;
            }
            Self::Push { value } => {
                _ = buffer.write_char(CMD_PUSH);
                buffer = value.compile_bytecode(buffer)?;
            }
            Self::Puts { value } => {
                _ = buffer.write_char(CMD_PUTS);
                buffer = value.compile_bytecode(buffer)?;
            }
            Self::Return { value } => {
                _ = buffer.write_char(CMD_RET);
                buffer = value.compile_bytecode(buffer)?;
            }
            Self::Scope { body } => {
                _ = buffer.write_char(CMD_SCOPE);
                buffer = body.compile_bytecode(buffer)?;
            }
            Self::Struct { name, body } => {
                _ = buffer.write_char(CMD_STRUCT);
                buffer = name.compile_bytecode(buffer)?;
                buffer = body.compile_bytecode(buffer)?;
            }
        }

        Ok(buffer)
    }
}

#[inline(always)]
pub fn fmt_body(f: &mut fmt::Formatter, body: &[Command]) -> fmt::Result {
    for (i, cmd) in body.iter().enumerate() {
        let ib = format!("\x1b[32m{i:03x}\x1b[0m");
        let cmd = format!("{cmd}");
        let mut cmd = cmd.split('\n').fold(String::new(), |mut buffer, c| {
            let _ = writeln!(buffer, ".{ib}{c}");
            buffer
        });

        if i == body.len() - 1 {
            cmd.pop();
        }

        f.write_str(&cmd)?;
    }

    Ok(())
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssignVariable { name, value } => {
                f.write_fmt(format_args!(": AssignVariable({name}, {value})"))
            }

            Self::Break => f.write_str(": Break"),

            Self::Builtin { name, args } => {
                f.write_fmt(format_args!(": Builtin({name}, {args:#?})"))
            }

            Self::Call { name, args } => {
                write!(f, ": Call({name}, {args:#?})")
            }

            Self::Conditional {
                condition,
                body,
                otherwise,
            } => {
                f.write_fmt(format_args!(": Conditional({condition}):\n"))?;

                fmt_body(f, body)?;

                if let Some(otherwise) = otherwise {
                    f.write_str("\n: Else:\n")?;
                    fmt_body(f, otherwise)?;
                }

                Ok(())
            }

            Self::DeclareVariable { name, value, kind } => {
                f.write_fmt(format_args!(": DeclareVariable({kind}, {name}, {value})"))
            }

            Self::For {
                var,
                iterator,
                body,
            } => {
                writeln!(f, ": For({var} in {iterator})")?;

                fmt_body(f, body)
            }

            Self::Function {
                name,
                args: _,
                ret,
                body,
            } => {
                writeln!(f, ": Function {name}(...) {ret}")?;

                fmt_body(f, body)
            }

            Self::Loop { body } => {
                f.write_str(": Loop:\n")?;

                fmt_body(f, body)
            }

            Self::Meta { pos, code } => {
                write!(f, ": Meta({pos:?}, {code:?})")
            }
            Self::MetaFile(file_name) => {
                write!(f, ": MetaFile({file_name:?})")
            }

            Self::Push { value } => write!(f, ": Push({value})"),
            Self::Puts { value } => write!(f, ": Puts({value})"),
            Self::Return { value } => write!(f, ": Return {value}"),

            Self::Scope { body } => {
                writeln!(f, ": Scope:")?;

                fmt_body(f, body)
            }
            Self::Struct { name, body } => write!(f, ": Struct {name} {body:#?}"),
        }
    }
}
