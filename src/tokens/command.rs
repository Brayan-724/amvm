use crate::parser::ParserResult;
use crate::{
    parser, CommandExpression, Compilable, Parser, Value, VariableKind, COMMAND_SEPARATOR, VAR_MUT,
    VAR_VAR,
};
use crate::{VAR_CONST, VAR_LET};
use std::fmt;

use super::AmvmType;

pub static CMD_DCLR_VAR: char = '\x51';
pub static CMD_ASGN_VAR: char = '\x52';
pub static CMD_PUTS: char = '\x53';
pub static CMD_EVAL: char = '\x54';
pub static CMD_SCOPE: char = '\x55';
pub static CMD_LOOP: char = '\x56';
pub static CMD_COND: char = '\x57';
pub static CMD_BREAK: char = '\x58';
pub static CMD_BUILTIN: char = '\x59';
pub static CMD_STRUCT: char = '\x5A';

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    AssignVariable {
        name: String,
        value: CommandExpression,
    },
    Break,
    Builtin {
        name: String,
        args: Vec<CommandExpression>,
    },
    Conditional {
        condition: CommandExpression,
        body: Vec<Command>,
        otherwise: Option<Vec<Command>>,
    },
    DeclareVariable {
        kind: VariableKind,
        name: String,
        value: CommandExpression,
    },
    Evaluate {
        expr: CommandExpression,
    },
    Loop {
        body: Vec<Command>,
    },
    Puts {
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
    pub fn get_kind(&self) -> char {
        match self {
            Self::AssignVariable { .. } => CMD_ASGN_VAR,
            Self::Break { .. } => CMD_BREAK,
            Self::Builtin { .. } => CMD_BUILTIN,
            Self::Conditional { .. } => CMD_COND,
            Self::DeclareVariable { .. } => CMD_DCLR_VAR,
            Self::Evaluate { .. } => CMD_EVAL,
            Self::Loop { .. } => CMD_LOOP,
            Self::Puts { .. } => CMD_PUTS,
            Self::Scope { .. } => CMD_SCOPE,
            Self::Struct { .. } => CMD_STRUCT,
        }
    }

    pub fn visit_asgn<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, name) = Value::visit_string(parser)?;
        let name = name.to_owned();
        let (parser, value) = CommandExpression::visit(parser)?;

        Ok((parser, Command::AssignVariable { name, value }))
    }

    pub fn visit_var<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
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
        let name = name.to_owned();
        let (parser, value) = CommandExpression::visit(parser)?;

        Ok((parser, Command::DeclareVariable { name, kind, value }))
    }

    fn visit_scope<'a>(parser: Parser<'a>) -> ParserResult<'a, Vec<Self>> {
        let mut body: Vec<Command> = vec![];
        let mut parser = parser;
        loop {
            let (_parser, v) = parser::opt(parser::char(COMMAND_SEPARATOR))(parser)?;
            parser = _parser;

            if v.is_some() {
                break;
            }

            let (_parser, cmd) = Command::visit(parser)?;
            parser = _parser;

            body.push(cmd);
        }

        Ok((parser, body))
    }

    pub fn visit<'a>(parser_: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, b) = parser::anychar(parser_)
            .map_err(parser_.nom_err_with_context("Expected command kind"))?;

        let (parser, value) = match b {
            _ if b == CMD_BREAK => (parser, Command::Break),
            _ if b == CMD_ASGN_VAR => Self::visit_asgn(parser)?,
            _ if b == CMD_DCLR_VAR => Self::visit_var(parser)?,
            _ if b == CMD_PUTS => {
                let (parser, value) = CommandExpression::visit(parser)?;
                (parser, Command::Puts { value })
            }
            _ if b == CMD_SCOPE => {
                let (parser, body) = Self::visit_scope(parser)?;
                (parser, Command::Scope { body })
            }
            _ if b == CMD_STRUCT => {
                let (parser, name) = Value::visit_string(parser)?;
                let name = Box::from(name);
                let (parser, fields_len) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected fields length"))?;

                let fields_len = fields_len as u8;
                let mut fields = Vec::with_capacity(fields_len as usize);

                let mut parser = parser;
                for _ in 0..fields_len {
                    let (parser_, field) = Value::visit_string(parser)?;
                    let (parser_, r#type) = AmvmType::visit(parser_)?;
                    parser = parser_;
                    fields.push((Box::from(field), r#type))
                }

                (parser, Command::Struct { name, body: fields })
            }
            _ if b == CMD_LOOP => {
                let (parser, body) = Self::visit_scope(parser)?;
                (parser, Command::Loop { body })
            }
            _ if b == CMD_COND => {
                let (parser, condition) = CommandExpression::visit(parser)?;
                let (parser, body) = Self::visit_scope(parser)?;
                let (parser, otherwise) = if parser.peek(0) != Some(COMMAND_SEPARATOR) {
                    let (parser, otherwise) = Self::visit_scope(parser)?;
                    (parser, Some(otherwise))
                } else {
                    let (_, parser) = parser::take(1usize)(parser)?;
                    (parser, None)
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
            _ if b == CMD_BUILTIN => {
                let (parser, name) = Value::visit_string(parser)?;
                let name = name.to_owned();
                let (parser, args_len) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Can't get arguments length"))?;
                let args_len = args_len as u8;

                let mut args = vec![];
                let mut parser = parser;

                for _ in 0..args_len {
                    let (parser_, arg) = CommandExpression::visit(parser)?;
                    parser = parser_;

                    args.push(arg);
                }

                (parser, Command::Builtin { name, args })
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
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(match self {
            Self::DeclareVariable { name, value, kind } => {
                let kind = kind.compile_bytecode();
                let name = Value::compile_string(name);
                let value = value.compile_bytecode();

                format!("{CMD_DCLR_VAR}{kind}{name}{value}")
            }
            Self::AssignVariable { name, value } => {
                let name = Value::compile_string(name);
                let value = value.compile_bytecode();
                format!("{CMD_ASGN_VAR}{name}{value}")
            }
            Self::Puts { value } => {
                let value = value.compile_bytecode();
                format!("{CMD_PUTS}{value}")
            }
            Self::Scope { body } => {
                let value = body.compile_bytecode();
                format!("{CMD_SCOPE}{value}{COMMAND_SEPARATOR}")
            }
            Self::Loop { body } => {
                let value = body.compile_bytecode();
                format!("{CMD_LOOP}{value}{COMMAND_SEPARATOR}")
            }
            Self::Conditional {
                condition,
                body,
                otherwise,
            } => {
                let condition = condition.compile_bytecode();
                let body = body.compile_bytecode();
                let otherwise = if let Some(otherwise) = otherwise {
                    let otherwise = otherwise.compile_bytecode();
                    format!("{otherwise}{COMMAND_SEPARATOR}")
                } else {
                    COMMAND_SEPARATOR.to_string()
                };

                format!("{CMD_COND}{condition}{body}{COMMAND_SEPARATOR}{otherwise}")
            }
            Self::Break => CMD_BREAK.to_string(),
            Self::Builtin { name, args } => {
                let name = Value::compile_string(name);

                let args_len = args.len() as u8 as char;
                let args: String = args.iter().map(|v| v.compile_bytecode()).collect();

                format!("{CMD_BUILTIN}{name}{args_len}{args}")
            }
            Self::Struct { name, body } => format!(
                "{CMD_STRUCT}{}{}",
                name.compile_bytecode(),
                body.compile_bytecode()
            ),

            _ => todo!("{self:#?}"),
        })
    }
}

#[inline(always)]
pub fn fmt_body(f: &mut fmt::Formatter, body: &Vec<Command>) -> fmt::Result {
    for (i, cmd) in body.iter().enumerate() {
        let ib = format!("\x1b[32m{i:03x}\x1b[0m");
        let cmd = format!("{cmd}");
        let mut cmd = cmd
            .split('\n')
            .map(|c| format!(".{ib}{c}\n"))
            .collect::<String>();

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
            Self::Break => f.write_str(": Break"),
            Self::Builtin { name, args } => {
                f.write_fmt(format_args!(": Builtin({name}, {args:#?})"))
            }
            Self::AssignVariable { name, value } => {
                f.write_fmt(format_args!(": AssignVariable({name}, {value})"))
            }
            Self::DeclareVariable { name, value, kind } => {
                f.write_fmt(format_args!(": DeclareVariable({kind}, {name}, {value})"))
            }
            Self::Evaluate { expr } => f.write_fmt(format_args!(": Evaluate({expr})")),
            Self::Puts { value } => f.write_fmt(format_args!(": Puts({value})")),
            Self::Scope { body } => {
                f.write_str(": Scope:\n")?;

                fmt_body(f, body)
            }
            Self::Struct { name, body } => f.write_fmt(format_args!(": Struct {name} {body:#?}")),
            Self::Loop { body } => {
                f.write_str(": Loop:\n")?;

                fmt_body(f, body)
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
        }
    }
}
