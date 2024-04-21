use crate::aml3::{Aml3Struct, Aml3Type};
use crate::{parser, Command, Parser, ParserResult, VariableKind};

use super::{Aml3Expr, Aml3Scope, Aml3Variable};

pub struct Aml3Command;

impl Aml3Command {
    pub fn visit_conditional<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, _) = parser::char(' ')(parser)?;
        let (parser, condition) = Aml3Expr::visit(parser)?;
        let (parser, _) = parser::char(' ')(parser)?;
        let (parser, body) = Aml3Scope::visit(parser, true)?;

        // TODO: Error
        let (parser, _) = parser::opt(parser::char(' '))(parser)?;

        let (parser, otherwise) = if let Ok((parser, _)) = parser::char::<_, ()>('@')(parser) {
            let (parser, maybe_else) = parser::take_until_space(parser)
                .map_err(parser.nom_err_with_context("Unexpected EOF"))?;

            match maybe_else.value {
                "else" => {
                    let (parser, _) = parser::char(' ')(parser)?;
                    let (parser, v) = Aml3Scope::visit(parser, true)?;

                    (parser, Some(v))
                }
                _ => {
                    return Err(parser.error(
                        parser::VerboseErrorKind::Context("Expected 'else' command"),
                        true,
                    ))
                }
            }
        } else {
            (parser, None)
        };

        Ok((
            parser,
            Command::Conditional {
                condition,
                body,
                otherwise,
            },
        ))
    }

    fn visit_command<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, cmd) = parser::take_until_space(parser)
            .map_err(parser.nom_err_with_context("Expected command"))?;

        tracing::trace!(?cmd.value);

        match cmd.value {
            "break" => Ok((parser, Command::Break)),
            "builtin" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (mut parser, name) = parser::take_until_space(parser)?;
                let name = name.value.to_owned();

                let mut args = vec![];

                while !parser.is_eol() {
                    let (_parser, _) = parser::char(' ')(parser)?;
                    let (_parser, expr) = Aml3Expr::visit(_parser)?;
                    parser = _parser;

                    args.push(expr);
                }

                Ok((parser, Command::Builtin { name, args }))
            }

            "declare" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, kind) = if let Some('$') = parser.peek(0) {
                    (parser, VariableKind::Const)
                } else {
                    let (parser, kind) = parser::needs_space(parser::take_until_space)(parser)?;
                    let kind = kind.value;

                    (
                        parser,
                        VariableKind::from_str(kind).ok_or_else(|| {
                            parser.error(
                                parser::VerboseErrorKind::Context("Unknown variable kind"),
                                true,
                            )
                        })?,
                    )
                };
                tracing::trace!(?kind);
                let (parser, name) = Aml3Variable::visit(parser)?;
                tracing::trace!(?name);
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, value) = Aml3Expr::visit(parser)?;
                tracing::trace!(?value);

                Ok((
                    parser,
                    Command::DeclareVariable {
                        kind,
                        name: name.to_owned(),
                        value,
                    },
                ))
            }

            "if" => Self::visit_conditional(parser),

            "loop" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, body) = Aml3Scope::visit(parser, true)?;

                Ok((parser, Command::Loop { body }))
            }

            "puts" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, value) = Aml3Expr::visit(parser)?;

                Ok((parser, Command::Puts { value }))
            }

            "struct" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, name) = Aml3Type::visit_name(parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, def) = Aml3Struct::visit_decl_block(parser)?;

                let name = Box::from(name.unwrap_or_default());
                let body = def.into_iter().map(|v| (Box::from(v.0), v.1)).collect();

                Ok((parser, Command::Struct { name, body }))
            }

            _ => Err(parser.error(parser::VerboseErrorKind::Context("Unknown command"), true)),
        }
    }

    fn visit_asgn<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, var) = Aml3Variable::visit(parser)?;

        let (parser, _) = parser::char(' ')(parser)?;

        let (parser, value) = Aml3Expr::visit(parser)?;

        Ok((
            parser,
            Command::AssignVariable {
                name: var.to_owned(),
                value,
            },
        ))
    }

    #[tracing::instrument("visit_command", fields(parser = &parser.value.get(..10).unwrap_or(&parser.value)), level = tracing::Level::TRACE)]
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (consumed_parser, command) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected command"))?;

        tracing::trace!(?command);

        match command {
            '@' => Self::visit_command(consumed_parser),
            '=' => Self::visit_asgn(consumed_parser),
            '{' => {
                let (parser, body) = Aml3Scope::visit(parser, true)?;

                Ok((parser, Command::Scope { body }))
            }
            _ => Err(parser.error(parser::VerboseErrorKind::Context("Unknown command"), true)),
        }
    }
}
