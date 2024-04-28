use crate::parser::{self, Parser, ParserResult};

pub struct Aml3Variable;

impl Aml3Variable {
    pub fn visit_ident<'a>(parser: Parser<'a>) -> ParserResult<'a, &str> {
        let (parser, name) = parser::take_until_space(parser)
            .map_err(parser.nom_err_with_context("Expected a space after a variable name"))?;

        Ok((parser, name.value))
    }
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, &str> {
        let (parser, _) = parser::char('$')(parser)?;

        Self::visit_ident(parser)
    }
}
