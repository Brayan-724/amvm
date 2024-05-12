use crate::aml3::Aml3Meta;
use crate::{
    parser::{self, Parser, ParserResult},
    tokens::Command,
};

use super::Aml3Command;

pub struct Aml3Scope;

impl Aml3Scope {
    pub fn visit(parser: Parser<'_>, outer: bool) -> ParserResult<'_, Vec<Command>> {
        let parser = if outer {
            parser::char('{')(parser)?.0
        } else {
            parser
        };

        let mut cmds = vec![];
        let mut has_meta = false;

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

            if parser.peek(0) == Some('!') {
                if has_meta {
                    eprintln!(
                        "Warning: Double meta definition at {:?}",
                        parser.cursor_position()
                    );
                }

                let (_parser, meta) = Aml3Meta::visit(parser)?;
                has_meta = if let Command::MetaFile(..) = &meta {
                    false
                } else {
                    true
                };
                cmds.push(meta);
                parser = _parser;
                continue;
            } else if std::env::var("AML3_DEBUG").is_ok() {
                let pos = parser.cursor_position();
                let (code, _) = parser::take_until("\n")(parser)?;

                cmds.push(Command::Meta {
                    pos: (pos.0 as u16, 0),
                    code: code.value.into(),
                });
            }

            let (_parser, cmd) = Aml3Command::visit(parser)?;
            cmds.push(cmd);
            has_meta = false;

            parser = _parser;
        }

        Ok((parser, cmds))
    }
}
