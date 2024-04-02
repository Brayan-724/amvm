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

    pub fn visit_command<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, cmd) = parser::take_until_space(parser)
            .map_err(parser.nom_err_with_context("Expected command"))?;

        println!("VISIT Command: {}", cmd.value);

        match cmd.value {
            "let" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, name) = Aml3Variable::visit(parser)?;
                println!("VISIT Command Let: {}", name);
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, value) = Aml3Expr::visit(parser)?;
                println!("VISIT Command Let: {}", value);

                Ok((
                    parser,
                    Command::DeclareVariable {
                        kind: VariableKind::Let,
                        name,
                        value,
                    },
                ))
            }
            "const" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, name) = Aml3Variable::visit(parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, value) = Aml3Expr::visit(parser)?;

                Ok((
                    parser,
                    Command::DeclareVariable {
                        kind: VariableKind::Const,
                        name,
                        value,
                    },
                ))
            }

            "loop" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, body) = Aml3Scope::visit(parser, true)?;

                Ok((parser, Command::Loop { body }))
            }

            "break" => Ok((parser, Command::Break)),
            "if" => Self::visit_conditional(parser),

            "puts" => {
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, value) = Aml3Expr::visit(parser)?;

                Ok((parser, Command::Puts { value }))
            }

            _ => todo!("{}", cmd.value),
        }
    }

    pub fn visit_asgn<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, var) = Aml3Variable::visit(parser)?;

        let (parser, _) = parser::char(' ')(parser)?;

        let (parser, value) = Aml3Expr::visit(parser)?;

        Ok((parser, Command::AssignVariable { name: var, value }))
    }

    // pub fn visit_expr<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {}

    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (consumed_parser, command) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected command"))?;

        println!("VISIT: {command}");

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
