use std::fmt;

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
    CMD_EVAL,
    CMD_FN,
    CMD_FOR,
    CMD_LOOP,
    CMD_PUTS,
    CMD_RET,
    CMD_SCOPE,
    CMD_STRUCT
}

#[derive(Debug, Clone, PartialEq)]
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

    Evaluate {
        expr: CommandExpression,
    },

    For {
        var: Box<str>,
        iterator: CommandExpression,
        body: Vec<Command>,
    },

    Function {
        name: Box<str>,
        args: Vec<(Box<str>, AmvmType)>,
        ret: AmvmType,
        body: Vec<Command>,
    },

    Loop {
        body: Vec<Command>,
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
    pub fn visit_asgn<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
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
            _ if b == CMD_ASGN_VAR => Self::visit_asgn(parser)?,

            _ if b == CMD_BREAK => (parser, Command::Break),

            _ if b == CMD_BUILTIN => {
                let (parser, name) = Value::visit_string(parser)?;
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

                (parser, Command::Call { name, args })
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

            _ if b == CMD_DCLR_VAR => Self::visit_var(parser)?,

            _ if b == CMD_FN => {
                let (parser, name) = Value::visit_string(parser)?;
                let name: Box<str> = name.into();

                let (parser, args_len) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Can't get arguments length"))?;
                let args_len = args_len as u8;

                let mut args = vec![];
                let mut parser = parser;

                for _ in 0..args_len {
                    let (parser_, name) = Value::visit_string(parser)?;
                    let (parser_, ty) = AmvmType::visit(parser_)?;
                    parser = parser_;

                    args.push((Box::from(name), ty));
                }

                let (parser, ret) = AmvmType::visit(parser)?;
                let (parser, body) = Self::visit_scope(parser)?;

                (
                    parser,
                    Command::Function {
                        name,
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

            _ if b == CMD_PUTS => {
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
            Self::AssignVariable { name, value } => {
                let name = Value::compile_string(name);
                let value = value.compile_bytecode();
                format!("{CMD_ASGN_VAR}{name}{value}")
            }
            Self::Break => CMD_BREAK.to_string(),
            Self::Builtin { name, args } => {
                let name = name.compile_bytecode();

                let args_len = args.len() as u8 as char;
                let args: String = args.iter().map(|v| v.compile_bytecode()).collect();

                format!("{CMD_BUILTIN}{name}{args_len}{args}")
            }
            Self::Call { name, args } => {
                let name = name.compile_bytecode();
                let args_len = args.len() as u8 as char;
                let args: String = args.iter().map(|v| v.compile_bytecode()).collect();

                format!("{CMD_CALL}{name}{args_len}{args}")
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
            Self::DeclareVariable { name, value, kind } => {
                let kind = kind.compile_bytecode();
                let name = name.compile_bytecode();
                let value = value.compile_bytecode();

                format!("{CMD_DCLR_VAR}{kind}{name}{value}")
            }
            Self::For {
                var,
                iterator,
                body,
            } => {
                let var = var.compile_bytecode();
                let iterator = iterator.compile_bytecode();
                let body = body.compile_bytecode();

                format!("{CMD_FOR}{var}{iterator}{body}{COMMAND_SEPARATOR}")
            }
            Self::Function {
                name,
                args,
                ret,
                body,
            } => {
                let name = name.compile_bytecode();
                let args_len = args.len() as u8 as char;
                let args: String = args
                    .iter()
                    .map(|v| {
                        format!(
                            "{v}{ty}",
                            v = v.0.compile_bytecode(),
                            ty = v.1.compile_bytecode(),
                        )
                    })
                    .collect();
                let ret = ret.compile_bytecode();
                let body = body.compile_bytecode();

                format!("{CMD_FN}{name}{args_len}{args}{ret}{body}{COMMAND_SEPARATOR}")
            }
            Self::Loop { body } => {
                let value = body.compile_bytecode();
                format!("{CMD_LOOP}{value}{COMMAND_SEPARATOR}")
            }
            Self::Puts { value } => {
                let value = value.compile_bytecode();
                format!("{CMD_PUTS}{value}")
            }
            Self::Return { value } => {
                let value = value.compile_bytecode();
                format!("{CMD_RET}{value}")
            }
            Self::Scope { body } => {
                let value = body.compile_bytecode();
                format!("{CMD_SCOPE}{value}{COMMAND_SEPARATOR}")
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

            Self::Evaluate { expr } => f.write_fmt(format_args!(": Evaluate({expr})")),

            Self::For {
                var,
                iterator,
                body,
            } => {
                write!(f, ": For({var} in {iterator})\n")?;

                fmt_body(f, body)
            }

            Self::Function {
                name,
                args: _,
                ret,
                body,
            } => {
                write!(f, ": Function {name}(...) {ret}\n")?;

                fmt_body(f, body)
            }

            Self::Loop { body } => {
                f.write_str(": Loop:\n")?;

                fmt_body(f, body)
            }

            Self::Puts { value } => f.write_fmt(format_args!(": Puts({value})")),

            Self::Return { value } => f.write_fmt(format_args!(": Return {value}")),

            Self::Scope { body } => {
                f.write_str(": Scope:\n")?;

                fmt_body(f, body)
            }
            Self::Struct { name, body } => f.write_fmt(format_args!(": Struct {name} {body:#?}")),
        }
    }
}
