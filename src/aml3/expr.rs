use crate::aml3::{Aml3Struct, Aml3Type};
use crate::{parser, BinaryConditionKind, CommandExpression, Parser, ParserResult};

use super::{Aml3Value, Aml3Variable};

pub struct Aml3Expr;

macro_rules! impl_op {
    (@common $kind:literal, $parser:ident, $command:ident) => {{
        tracing::trace!(kind = $kind, command = stringify!($command));
        let (parser, _) = parser::char(' ')($parser)?;
        let (parser, a) = Aml3Expr::visit(parser)?;
        let (parser, _) = parser::char(' ')(parser)?;
        let (parser, b) = Aml3Expr::visit(parser)?;

        (parser, a, b)
    }};

    (@single $parser:ident, $command:ident) => {{
        let (parser, a, b) = impl_op!(@common "single", $parser, $command);

        Ok((parser, CommandExpression::$command(a.into(), b.into())))
    }};

    (@binary $parser:ident, $command:ident) => {{
        let (parser, a, b) = impl_op!(@common "binary", $parser, $command);

        Ok((parser, CommandExpression::BinaryCondition(
            BinaryConditionKind::$command, a.into(), b.into()
        )))
    }};
}

impl Aml3Expr {
    #[tracing::instrument("visit_expr", fields(parser = &parser.value.get(..10).unwrap_or(&parser.value)), level = tracing::Level::TRACE)]
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, CommandExpression> {
        let (consumed_parser, kind) =
            parser::anychar(parser).map_err(parser.nom_err_with_context("Expected operator"))?;

        tracing::trace!(expr.char = ?kind);

        match kind {
            '+' => impl_op!(@single consumed_parser, Addition),
            '-' => impl_op!(@single consumed_parser, Substraction),
            '.' => impl_op!(@single consumed_parser, Property),

            '#' => {
                let (parser, name) = Aml3Type::visit(consumed_parser)?;
                let (parser, _) = parser::char(' ')(parser)?;
                let (parser, decl) = Aml3Struct::visit_def_block(parser)?;
                let decl = decl.into_iter().map(|v| (Box::from(v.0), v.1)).collect();

                Ok((parser, CommandExpression::Struct(name, decl)))
            }

            '$' => {
                let (parser, var) = Aml3Variable::visit(parser)?;
                Ok((parser, CommandExpression::Var(var.to_owned())))
            }

            '_' => Ok((consumed_parser, CommandExpression::Prev)),

            '!' | '>' | '<' | '=' => {
                let parser = consumed_parser;
                let (consumed_parser, second_kind) = parser::anychar(parser)
                    .map_err(parser.nom_err_with_context("Expected operator"))?;

                tracing::trace!(expr.char = ?format!("{kind}{second_kind}"));

                match (kind, second_kind) {
                    ('>', '=') => impl_op!(@binary consumed_parser, GreaterThanEqual),
                    ('>', _) => impl_op!(@binary parser, GreaterThan),

                    ('<', '=') => impl_op!(@binary consumed_parser, LessThanEqual),
                    ('<', _) => impl_op!(@binary parser, LessThan),

                    ('=', '=') => impl_op!(@binary consumed_parser, Equal),
                    ('!', '=') => impl_op!(@binary consumed_parser, NotEqual),
                    // TODO:
                    // ('!', ' ') => impl_op!(@single parser, Negate),
                    _ => Err(parser.error(
                        parser::VerboseErrorKind::Context("Expected boolean operator"),
                        true,
                    )),
                }
            }

            _ => {
                tracing::trace!("Expression Fallback");
                let (parser, val) = Aml3Value::visit(parser)?;

                Ok((parser, CommandExpression::Value(val)))
            }
        }
    }
}
