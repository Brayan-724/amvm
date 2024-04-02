use crate::{parser, Parser, ParserResult, Value};

pub struct Aml3Variable;

impl Aml3Variable {
    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Value> {
        let (parser, _) = parser::char('$')(parser)?;

        let (parser, name) = parser::take_until_space(parser)
            .map_err(parser.nom_err_with_context("Expected a space after a variable name"))?;

        Ok((parser, Value::String(name.value.to_owned())))
    }
}
