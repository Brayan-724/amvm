use crate::{
    aml3::{Aml3Expr, Aml3Variable},
    parser::{self, Parser, ParserResult},
    tokens::{AmvmType, CommandExpression},
};

use super::Aml3Type;

pub struct Aml3Struct;

impl Aml3Struct {
    pub fn visit_decl_block(parser: Parser<'_>) -> ParserResult<'_, Vec<(&str, AmvmType)>> {
        let (parser, _) = parser::char('{')(parser)?;

        let mut properties = vec![];

        let mut parser = parser;
        loop {
            if parser.value.is_empty() {
                return Err(parser.error(parser::VerboseErrorKind::Context("Expected '}'"), true));
            }

            let value = parser::take_space::<_, ()>(parser).ok();

            parser = if let Some((_parser, c)) = value {
                parser = if c == '\n' {
                    _parser.new_line()
                } else {
                    _parser
                };

                continue;
            } else {
                parser
            };

            let value = parser::char::<_, ()>('}')(parser).ok();
            if let Some((_parser, _)) = value {
                parser = _parser;
                break;
            }

            let at = parser.pointer_position();
            tracing::trace!("------- at {at:02x} -------");

            let (_parser, name) = Aml3Variable::visit_ident(parser)?;
            let (_parser, _) = parser::char(' ')(_parser)?;
            let (_parser, value) = Aml3Type::visit(_parser)?;
            parser = _parser;

            let property = (name, value);
            properties.push(property);
        }

        Ok((parser, properties))
    }

    pub fn visit_def_block(parser: Parser<'_>) -> ParserResult<'_, Vec<(&str, CommandExpression)>> {
        let (parser, _) = parser::char('{')(parser)?;

        let mut properties = vec![];

        let mut parser = parser;
        loop {
            if parser.value.is_empty() {
                return Err(parser.error(parser::VerboseErrorKind::Context("Expected '}'"), true));
            }

            let value = parser::take_space::<_, ()>(parser).ok();

            parser = if let Some((_parser, c)) = value {
                parser = if c == '\n' {
                    _parser.new_line()
                } else {
                    _parser
                };

                continue;
            } else {
                parser
            };

            let value = parser::char::<_, ()>('}')(parser).ok();
            if let Some((_parser, _)) = value {
                parser = _parser;
                break;
            }

            let at = parser.pointer_position();
            tracing::trace!("------- at {at:02x} -------");

            let (_parser, name) = Aml3Variable::visit_ident(parser)?;
            let (_parser, _) = parser::char(' ')(_parser)?;
            let (_parser, value) = Aml3Expr::visit(_parser)?;
            parser = _parser;

            let property = (name, value);
            properties.push(property);
        }

        Ok((parser, properties))
    }
}
