use crate::{BinaryConditionKind, CommandExpression};

use super::{Aml3Error, Aml3Parser, Aml3Value, Aml3Variable};

pub struct Aml3Expr;

impl Aml3Expr {
    pub fn visit(parser: &mut Aml3Parser) -> Result<CommandExpression, Aml3Error> {
        let kind = parser
            .peek(0)
            .ok_or_else(|| parser.error_expected("expression", None))?;

        match kind {
            '+' => {
                let _ = parser.consume();

                Ok(CommandExpression::Addition(
                    Box::from(Aml3Expr::visit(parser)?),
                    Box::from(Aml3Expr::visit(parser)?),
                ))
            }
            '-' => {
                let _ = parser.consume();

                Ok(CommandExpression::Substraction(
                    Box::from(Aml3Expr::visit(parser)?),
                    Box::from(Aml3Expr::visit(parser)?),
                ))
            }
            '$' => Ok(CommandExpression::Var(Aml3Variable::visit(parser)?)),
            '.' => {
                let _ = parser.consume();

                Ok(CommandExpression::Property(
                    Box::from(Aml3Expr::visit(parser)?),
                    Box::from(Aml3Expr::visit(parser)?),
                ))
            }

            '!' | '>' | '<' | '=' => {
                let sk = parser.peek(1);

                match (kind, sk) {
                    ('>', Some('=')) => {
                        let _ = parser.consume();
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::GreaterThanEqual,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    ('>', _) => {
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::GreaterThan,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    ('<', Some('=')) => {
                        let _ = parser.consume();
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::LessThanEqual,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    ('<', _) => {
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::LessThan,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    ('=', Some('=')) => {
                        let _ = parser.consume();
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::Equal,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    ('!', Some('=')) => {
                        let _ = parser.consume();
                        let _ = parser.consume();

                        Ok(CommandExpression::BinaryCondition(
                            BinaryConditionKind::NotEqual,
                            Box::from(Aml3Expr::visit(parser)?),
                            Box::from(Aml3Expr::visit(parser)?),
                        ))
                    }
                    _ => Err(parser
                        .error_expected("boolean expression", Some(format!("{kind} + {sk:?}")))),
                }
            }

            _ => Ok(CommandExpression::Value(Aml3Value::visit(parser)?)),
        }
    }
}
