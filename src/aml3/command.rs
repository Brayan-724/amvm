use crate::{
    aml3::{Aml3Expr, Aml3Scope, Aml3Struct, Aml3Type, Aml3Variable},
    parser::{self, Parser, ParserResult},
    tokens::{Command, CommandExpression, VariableKind},
};

pub struct Aml3Command;

impl Aml3Command {
    fn visit_args<'a>(mut parser: Parser<'a>) -> ParserResult<'a, Vec<CommandExpression>> {
        let mut args = vec![];

        while !parser.is_eol() {
            let (_parser, _) = parser::char(' ')(parser)?;
            let (_parser, expr) = Aml3Expr::visit(_parser)?;
            parser = _parser;

            args.push(expr);
        }

        Ok((parser, args))
    }

    pub fn visit_conditional<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
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

    fn visit_command<'a>(parser__: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser_, cmd) = parser::take_until_space(parser__)
            .map_err(parser__.nom_err_with_context("Expected command"))?;

        tracing::trace!(?cmd.value);

        let (parser, _) = parser::char(' ')(parser_)?;
        match cmd.value {
            "break" => Ok((parser_, Command::Break)),
            "builtin" => {
                let (parser, name) = parser::take_until_space(parser)?;
                let (parser, args) = Self::visit_args(parser)?;

                Ok((
                    parser,
                    Command::Builtin {
                        name: name.value.into(),
                        args,
                    },
                ))
            }

            "call" => {
                let (parser, name) = Aml3Expr::visit(parser)?;
                let (parser, args) = Self::visit_args(parser)?;

                Ok((parser, Command::Call { name, args }))
            }

            "declare" => {
                let (parser, kind) = if let Some('$') = parser.peek(0) {
                    (parser, VariableKind::Const)
                } else {
                    let (parser, kind) = parser::needs_space(parser::take_until_space)(parser)?;
                    let kind = kind.value;
                    let kind = VariableKind::from_str(kind).ok_or_else(|| {
                        parser.error(
                            parser::VerboseErrorKind::Context("Unknown variable kind"),
                            true,
                        )
                    })?;

                    (parser, kind)
                };
                tracing::trace!(?kind);
                let (parser, name) = parser::needs_space(Aml3Variable::visit)(parser)?;
                tracing::trace!(?name);
                let (parser, value) = Aml3Expr::visit(parser)?;
                tracing::trace!(?value);

                Ok((
                    parser,
                    Command::DeclareVariable {
                        name: name.into(),
                        kind,
                        value,
                    },
                ))
            }

            "if" => Self::visit_conditional(parser),

            "loop" => {
                let (parser, body) = Aml3Scope::visit(parser, true)?;

                Ok((parser, Command::Loop { body }))
            }

            "puts" => {
                let (parser, value) = Aml3Expr::visit(parser)?;

                Ok((parser, Command::Puts { value }))
            }

            "struct" => {
                let (parser, name) = parser::needs_space(Aml3Type::visit_name)(parser)?;
                let (parser, def) = Aml3Struct::visit_decl_block(parser)?;

                let name = Box::from(name.unwrap_or_default());
                let body = def.into_iter().map(|v| (Box::from(v.0), v.1)).collect();

                Ok((parser, Command::Struct { name, body }))
            }

            "fn" => {
                let (parser, ret) = parser::needs_space(Aml3Type::visit)(parser)?;
                tracing::trace!(?ret);

                let (parser, name) = Aml3Variable::visit(parser)?;
                tracing::trace!(?name);
                let name: Box<str> = name.into();

                let (parser, args) = {
                    let mut parser_out = parser;
                    let mut args = vec![];

                    loop {
                        let (parser, _) = parser::char(' ')(parser_out)?;
                        let (_, c) = parser::anychar(parser)?;

                        match c {
                            '$' => {
                                tracing::trace!(arg_index = args.len());

                                let (parser, name) = Aml3Variable::visit(parser)?;
                                tracing::trace!(?name);
                                let name: Box<str> = name.into();

                                let (parser, _) = parser::char(' ')(parser)?;

                                let (parser, ty) = Aml3Type::visit(parser)?;
                                tracing::trace!(?ty);

                                args.push((name, ty));

                                parser_out = parser;
                            }
                            '{' => break,
                            _ => {
                                return Err(parser.error(parser::VerboseErrorKind::Char('$'), true))
                            }
                        }
                    }

                    (parser_out, args)
                };

                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, body) = Aml3Scope::visit(parser, true)?;
                tracing::trace!(?body);

                Ok((
                    parser,
                    Command::Function {
                        name,
                        args,
                        ret,
                        body,
                    },
                ))
            }

            "for" => {
                let (parser, var) = parser::needs_space(Aml3Variable::visit)(parser)?;
                let (parser, iterator) = parser::needs_space(Aml3Expr::visit)(parser)?;
                let (parser, body) = Aml3Scope::visit(parser, true)?;

                Ok((
                    parser,
                    Command::For {
                        var: var.into(),
                        iterator,
                        body,
                    },
                ))
            }

            "ret" => {
                let (parser, expr) = Aml3Expr::visit(parser)?;

                Ok((parser, Command::Return { value: expr }))
            }

            _ => Err(parser__.error(parser::VerboseErrorKind::Context("Unknown command"), true)),
        }
    }

    fn visit_asgn<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, var) = parser::needs_space(Aml3Variable::visit)(parser)?;
        let (parser, value) = Aml3Expr::visit(parser)?;

        Ok((
            parser,
            Command::AssignVariable {
                name: var.into(),
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
            _ => Err(parser.error(parser::VerboseErrorKind::Char('@'), true)),
        }
    }
}
