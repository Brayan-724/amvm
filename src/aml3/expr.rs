use crate::{parser, BinaryConditionKind, CommandExpression, Parser, ParserResult};

use super::{Aml3Value, Aml3Variable};

pub struct Aml3Expr;

impl Aml3Expr {
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, CommandExpression> {
        let (consumed_parser, kind) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected operator"))?;

        match kind {
            '+' => {
                let (parser, a) = Aml3Expr::visit(consumed_parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, b) = Aml3Expr::visit(parser)?;

                Ok((parser, CommandExpression::Addition(a.into(), b.into())))
            }
            '-' => {
                let (parser, a) = Aml3Expr::visit(consumed_parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, b) = Aml3Expr::visit(parser)?;

                Ok((parser, CommandExpression::Substraction(a.into(), b.into())))
            }
            '$' => {
                let (parser, var) = Aml3Variable::visit(parser)?;
                Ok((parser, CommandExpression::Var(var)))
            }
            '.' => {
                let (parser, a) = Aml3Expr::visit(consumed_parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, b) = Aml3Expr::visit(parser)?;

                Ok((parser, CommandExpression::Property(a.into(), b.into())))
            }

            '!' | '>' | '<' | '=' => {
                let (double_consumed_parser, second_kind) = parser::anychar(consumed_parser)
                    .map_err(parser.nom_err_with_context("Expected operator"))?;

                match (kind, second_kind) {
                    ('>', '=') => {
                        let (parser, a) = Aml3Expr::visit(double_consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::GreaterThanEqual,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    ('>', _) => {
                        let (parser, a) = Aml3Expr::visit(consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::GreaterThan,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    ('<', '=') => {
                        let (parser, a) = Aml3Expr::visit(double_consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::LessThanEqual,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    ('<', _) => {
                        let (parser, a) = Aml3Expr::visit(consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::LessThan,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    ('=', '=') => {
                        let (parser, a) = Aml3Expr::visit(double_consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::Equal,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    ('!', '=') => {
                        let (parser, a) = Aml3Expr::visit(double_consumed_parser)?;
                        let (parser, _) = parser::char(' ')(parser)?;
                        let (parser, b) = Aml3Expr::visit(parser)?;

                        Ok((
                            parser,
                            CommandExpression::BinaryCondition(
                                BinaryConditionKind::NotEqual,
                                a.into(),
                                b.into(),
                            ),
                        ))
                    }
                    _ => Err(parser.error(
                        parser::VerboseErrorKind::Context("Expected boolean operator"),
                        true,
                    )),
                }
            }

            _ => {
                println!("VISIT Expr fallback: {:?}", kind);
                let (parser, val) = Aml3Value::visit(parser)?;

                Ok((parser, CommandExpression::Value(val)))
            }
        }
    }
}
