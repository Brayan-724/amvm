use crate::{
    parser::{self, Parser, ParserResult, CMD_VERBOSE},
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

    pub fn visit<'a>(parser: Parser<'a>) -> ParserResult<'a, Self> {
        let (parser, header) = AmvmHeader::visit(parser)?;

        let (_, parser) = parser::take(1usize)(parser)?;
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

            if CMD_VERBOSE {
                let ib = format!("\x1b[32m{at:03x}\x1b[0m");
                let cmd = format!("{cmd}");
                let mut cmd = cmd
                    .split('\n')
                    .map(|c| format!("{ib}{c}\n"))
                    .collect::<String>();

                cmd.pop();

                println!("{cmd}");
            }

            cmds.push(cmd);
        }

        Ok((parser, Self::new(header, cmds)))
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
