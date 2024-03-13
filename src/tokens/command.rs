use crate::error::error_msgs;
use crate::{
    CommandExpression, Compilable, Parser, ParserError, Value, VariableKind, COMMAND_SEPARATOR,
};
use crate::{VAR_CONST, VAR_LET};
use std::fmt;

pub static CMD_DCLR_VAR: char = '\x51';
pub static CMD_ASGN_VAR: char = '\x52';
pub static CMD_PUTS: char = '\x53';
pub static CMD_EVAL: char = '\x54';
pub static CMD_SCOPE: char = '\x55';
pub static CMD_LOOP: char = '\x56';
pub static CMD_COND: char = '\x57';
pub static CMD_BREAK: char = '\x58';

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Break,
    AssignVariable {
        name: Value,
        value: CommandExpression,
    },
    Conditional {
        condition: CommandExpression,
        body: Vec<Command>,
        otherwise: Option<Vec<Command>>,
    },
    DeclareVariable {
        kind: VariableKind,
        name: Value,
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
}

impl Command {
    pub fn get_kind(&self) -> char {
        match self {
            Self::AssignVariable { .. } => CMD_ASGN_VAR,
            Self::DeclareVariable { .. } => CMD_DCLR_VAR,
            Self::Evaluate { .. } => CMD_EVAL,
            Self::Puts { .. } => CMD_PUTS,
            Self::Scope { .. } => CMD_SCOPE,
            Self::Loop { .. } => CMD_LOOP,
            Self::Conditional { .. } => CMD_COND,
            Self::Break { .. } => CMD_BREAK,
        }
    }

    pub fn visit_asgn(parser: &mut Parser) -> Result<Self, ParserError> {
        let name = Value::visit(parser)?;
        let value = CommandExpression::visit(parser)?;

        Ok(Command::AssignVariable { name, value })
    }

    pub fn visit_var(parser: &mut Parser) -> Result<Self, ParserError> {
        let kind = parser.consume().ok_or_else(|| {
            parser.error_corrupt(
                error_msgs::ERROR_INVALID_CMD_DECL,
                "Can't get variable kind",
                1,
            )
        })?;
        let kind = match kind {
            b if b == VAR_CONST => VariableKind::Const,
            b if b == VAR_LET => VariableKind::Let,
            b => {
                return Err(parser.error_corrupt(
                    error_msgs::ERROR_UNKNOWN_VAR_KIND,
                    format!("{b:?}"),
                    1,
                ))
            }
        };

        let name = Value::visit(parser)?;
        let value = CommandExpression::visit(parser)?;

        Ok(Command::DeclareVariable { name, kind, value })
    }

    fn visit_scope(parser: &mut Parser) -> Result<Vec<Self>, ParserError> {
        let mut body: Vec<Command> = vec![];
        loop {
            if parser.peek(0) == Some(COMMAND_SEPARATOR) {
                parser.pointer += 1;
                break;
            }

            let cmd = Command::visit(parser)?;
            body.push(cmd);
        }

        Ok(body)
    }

    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        let b = parser.consume().ok_or_else(|| {
            parser.error_corrupt(
                error_msgs::ERROR_INVALID_CMD_DECL,
                "Can't get command kind",
                1,
            )
        })?;

        match b {
            b if b == CMD_BREAK => Ok(Command::Break),
            b if b == CMD_ASGN_VAR => Self::visit_asgn(parser),
            b if b == CMD_DCLR_VAR => Self::visit_var(parser),
            b if b == CMD_PUTS => Ok(Command::Puts {
                value: CommandExpression::visit(parser)?,
            }),
            b if b == CMD_SCOPE => Ok(Command::Scope {
                body: Self::visit_scope(parser)?,
            }),
            b if b == CMD_LOOP => Ok(Command::Loop {
                body: Self::visit_scope(parser)?,
            }),
            b if b == CMD_COND => Ok(Command::Conditional {
                condition: CommandExpression::visit(parser)?,
                body: Self::visit_scope(parser)?,
                otherwise: if parser.peek(0) != Some(COMMAND_SEPARATOR) {
                    Some(Self::visit_scope(parser)?)
                } else {
                    let _ = parser.consume();
                    None
                },
            }),
            _ => Err(parser.error_corrupt(
                error_msgs::ERROR_INVALID_CMD_DECL,
                format!("Unknown command {:02x?}", b as u8),
                1,
            )),
        }
    }
}
impl Compilable for Command {
    fn compile_bytecode(&self) -> Box<str> {
        match self {
            Self::DeclareVariable { name, value, kind } => {
                if !name.is_string() {
                    panic!("Variable name should be string");
                }
                let kind = kind.compile_bytecode();
                let name = name.compile_bytecode();
                let value = value.compile_bytecode();

                Box::from(format!("{CMD_DCLR_VAR}{kind}{name}{value}"))
            }
            Self::AssignVariable { name, value } => {
                if !name.is_string() {
                    panic!("Variable name should be string");
                }
                let name = name.compile_bytecode();
                let value = value.compile_bytecode();
                Box::from(format!("{CMD_ASGN_VAR}{name}{value}"))
            }
            Self::Puts { value } => {
                let value = value.compile_bytecode();
                Box::from(format!("{CMD_PUTS}{value}"))
            }
            Self::Scope { body } => {
                let value = body.compile_bytecode();
                Box::from(format!("{CMD_SCOPE}{value}{COMMAND_SEPARATOR}"))
            }
            Self::Loop { body } => {
                let value = body.compile_bytecode();
                Box::from(format!("{CMD_LOOP}{value}{COMMAND_SEPARATOR}"))
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

                Box::from(format!(
                    "{CMD_COND}{condition}{body}{COMMAND_SEPARATOR}{otherwise}"
                ))
            }
            Self::Break => Box::from(CMD_BREAK.to_string()),
            _ => todo!("{self:#?}"),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_body(f: &mut fmt::Formatter, body: &Vec<Command>) -> fmt::Result {
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

        match self {
            Self::Break => f.write_str(": Break"),
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
