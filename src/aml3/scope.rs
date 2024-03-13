use crate::Command;

use super::{Aml3Command, Aml3Error, Aml3Parser};

pub struct Aml3Scope;

impl Aml3Scope {
    pub fn visit(parser: &mut Aml3Parser, outer: bool) -> Result<Vec<Command>, Aml3Error> {
        if outer && !parser.consume_static('{') {
            return Err(parser.error_expected("\"{\"", None));
        }

        let mut cmds = vec![];

        loop {
            if parser.pointer >= parser.bytes.len() {
                if outer {
                    return Err(parser.error_expected("\"}\"", None));
                }
                break;
            }

            if outer && parser.consume_static('}') {
                break;
            }

            if parser.bytes[parser.pointer] == ' ' as u8 {
                parser.go_forward(1);
                continue;
            }

            if parser.bytes[parser.pointer] == '\n' as u8 {
                parser.go_forward(1);
                parser.new_line();
                continue;
            }

            cmds.push(Aml3Command::visit(parser)?);
        }

        Ok(cmds)
    }
}
