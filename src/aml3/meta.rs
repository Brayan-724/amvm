use crate::parser::{self, Parser, ParserResult};
use crate::tokens::Command;

pub struct Aml3Meta;

impl Aml3Meta {
    fn visit_file_meta<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, _) = parser::char('!')(parser)?;
        let (parser, _) = parser::char(' ')(parser)?;

        let (file_name, parser) = parser::take_until("\n")(parser)?;

        Ok((parser, Command::MetaFile(file_name.value.into())))
    }

    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Command> {
        let (parser, _) = parser::char('!')(parser)?;

        if parser.peek(0) == Some('!') {
            return Self::visit_file_meta(parser);
        }

        let (parser_, _) = parser::char(' ')(parser)?;

        let (parser, line) = parser::digit1(parser_)?;
        let line = line.value.parse::<u16>().map_err(|_| {
            parser_.error(parser::VerboseErrorKind::Context("Cannot parse line"), true)
        })?;
        let (parser_, _) = parser::char(':')(parser)?;
        let (parser, col) = parser::digit1(parser_)?;
        let col = col.value.parse::<u16>().map_err(|_| {
            parser_.error(parser::VerboseErrorKind::Context("Cannot parse line"), true)
        })?;
        let (parser, _) = parser::char(' ')(parser)?;

        let (code, parser) = parser::take_until("\n")(parser)?;

        let pos = (line, col);
        let code = Box::from(code.value);

        Ok((parser, Command::Meta { pos, code }))
    }
}
