use crate::{Command, VariableKind};

use super::{Aml3Error, Aml3Expr, Aml3Parser, Aml3Scope, Aml3Variable};

pub struct Aml3Command;

impl Aml3Command {
    pub fn visit_conditional(parser: &mut Aml3Parser) -> Result<Command, Aml3Error> {
        let condition = Aml3Expr::visit(parser)?;
        let body = Aml3Scope::visit(parser, true)?;

        parser.consume_static(' ');

        let otherwise = if parser.consume_static('@') {
            let maybe_else = parser
                .consume_until(' ')
                .ok_or_else(|| parser.error_expected("else", None))?;

            match &maybe_else as &str {
                "else" => {
                    parser.consume_static(' ');

                    Some(Aml3Scope::visit(parser, true)?)
                }
                _ => return Err(parser.error_expected("else", Some(maybe_else))),
            }
        } else {
            None
        };

        Ok(Command::Conditional {
            condition,
            body,
            otherwise,
        })
    }

    pub fn visit_command(parser: &mut Aml3Parser) -> Result<Command, Aml3Error> {
        let cmd = parser
            .consume_until(' ')
            .ok_or_else(|| parser.error_expected("keyword", None))?;

        match &cmd as &str {
            "let" => Ok(Command::DeclareVariable {
                kind: VariableKind::Let,
                name: Aml3Variable::visit(parser)?,
                value: Aml3Expr::visit(parser)?,
            }),
            "const" => Ok(Command::DeclareVariable {
                kind: VariableKind::Const,
                name: Aml3Variable::visit(parser)?,
                value: Aml3Expr::visit(parser)?,
            }),

            "loop" => Ok(Command::Loop {
                body: Aml3Scope::visit(parser, true)?,
            }),

            "break" => Ok(Command::Break),

            "if" => Self::visit_conditional(parser),

            "puts" => Ok(Command::Puts {
                value: Aml3Expr::visit(parser)?,
            }),

            _ => todo!("{cmd}"),
        }
    }

    pub fn visit_asgn(parser: &mut Aml3Parser) -> Result<Command, Aml3Error> {
        let var = Aml3Variable::visit(parser)?;

        parser.consume_static(' ');

        let value = Aml3Expr::visit(parser)?;

        Ok(Command::AssignVariable { name: var, value })
    }

    // pub fn visit_expr(parser: &mut Aml3Parser) -> Result<Command, Aml3Error> {}

    pub fn visit(parser: &mut Aml3Parser) -> Result<Command, Aml3Error> {
        let command = parser
            .consume()
            .ok_or_else(|| parser.error_expected("command kind", None))?;

        match command {
            '@' => Self::visit_command(parser),
            '=' => Self::visit_asgn(parser),
            '{' => {
                parser.go_back(1);
                Ok(Command::Scope {
                    body: Aml3Scope::visit(parser, true)?,
                })
            }
            _ => Err(parser.error(format!("Unknown command kind: {command:?}"))),
        }
    }
}
