use crate::parser::CMD_VERBOSE;
use crate::{AmvmHeader, Command, Compilable, Parser, ParserError, Runtime};

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

    pub fn visit(parser: &mut Parser) -> Result<Self, ParserError> {
        let header = AmvmHeader::visit(parser)?;

        let mut cmds = vec![];
        loop {
            if parser.pointer >= parser.bytes.len() {
                break;
            }

            let at = parser.pointer;
            let cmd = Command::visit(parser)?;
            if CMD_VERBOSE {
                println!("{at}: {cmd}");
            }
            cmds.push(cmd);
        }

        Ok(Self::new(header, cmds))
    }

    pub fn runtime(self) -> Runtime {
        Runtime::new(self.header, self.body)
    }
}

impl Compilable for Program {
    fn compile_bytecode(&self) -> Box<str> {
        Box::from(format!(
            "{header}{body}",
            header = self.header.compile_bytecode(),
            body = self.body.compile_bytecode()
        ))
    }
}
