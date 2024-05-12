use std::fmt::Write;

use crate::CompileResult;
use crate::{
    parser::{self, Parser, ParserResult},
    runtime::Runtime,
    tokens::{AmvmHeader, Command},
    Compilable,
};

pub struct Program {
    pub header: AmvmHeader,
    pub body: Vec<Command>,
}

impl Program {
    pub fn new(header: AmvmHeader, body: impl Into<Vec<Command>>) -> Self {
        Self {
            header,
            body: body.into(),
        }
    }

    pub fn visit(parser: Parser<'_>) -> ParserResult<'_, Self> {
        let (parser, header) = AmvmHeader::visit(parser)?;

        let (parser, _) = parser::anychar(parser)?;
        let parser = parser.new_line();

        let mut cmds = vec![];
        let mut parser = parser;
        loop {
            if parser.value.is_empty() {
                break;
            }

            let at = parser.pointer_position();
            let (_parser, cmd) = Command::visit(parser)?;
            parser = _parser;
            parser = parser.new_line();

            if std::env::var("CMD_VERBOSE").is_ok() {
                let ib = format!("\x1b[32m{at:03x}\x1b[0m");
                let cmd = format!("{cmd}");
                let mut cmd = cmd.split('\n').fold(String::new(), |mut buffer, c| {
                    let _ = writeln!(buffer, "{ib}{c}");
                    buffer
                });

                cmd.pop();

                println!("{cmd}");
            }

            cmds.push(cmd);
        }

        Ok((parser, Self::new(header, cmds)))
    }

    pub fn runtime(self, filename: Box<str>) -> Runtime {
        Runtime::new(filename, self.header, self.body)
    }
}

impl Compilable for Program {
    fn compile_bytecode(&self, mut buffer: String) -> CompileResult {
        buffer = self.header.compile_bytecode(buffer)?;
        buffer = self.body.compile_bytecode(buffer)?;

        Ok(buffer)
    }
}
