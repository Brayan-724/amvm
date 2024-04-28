use crate::{
    parser::{self, Parser, ParserResult},
    tokens::Command,
};

use super::Aml3Command;

pub struct Aml3Scope;

impl Aml3Scope {
    pub fn visit<'a>(parser: Parser<'a>, outer: bool) -> ParserResult<'a, Vec<Command>> {
        let parser = if outer {
            parser::char('{')(parser)?.0
        } else {
            parser
        };

        let mut cmds = vec![];

        let mut parser = parser;
        loop {
            if parser.value.is_empty() {
                if outer {
                    return Err(
                        parser.error(parser::VerboseErrorKind::Context("Expected '}'"), true)
                    );
                }

                break;
            }

            if outer {
                let value = parser::char::<_, ()>('}')(parser).ok();
                if let Some((_parser, _)) = value {
                    parser = _parser;
                    break;
                }
            }

            {
                let value = parser::char::<_, ()>(';')(parser).ok();
                if let Some((_parser, _)) = value {
                    let (_parser, _) =
                        parser::take_while(|i: char| !(i == '\n' || i == '\r'))(_parser)?;
                    parser = _parser;
                    continue;
                }
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

            let at = parser.pointer_position();
            tracing::trace!("------- at {at:02x} -------");

            let (_parser, cmd) = Aml3Command::visit(parser)?;
            parser = _parser;

            cmds.push(cmd);
        }

        Ok((parser, cmds))
    }
}
