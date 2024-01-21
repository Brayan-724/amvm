use crate::{
    Command, CommandExpression, Runtime, Value, VariableKind, AMVM_HEADER, CMD_DCLR_VAR, CMD_PUTS,
    EXPR_VALUE, EXPR_VAR, VALUE_F32, VALUE_I16, VALUE_STRING, VALUE_U8, VALUE_UNDEFINED, VAR_CONST,
    VAR_LET,
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Parser {
    bytes: Box<[u8]>,
    pointer: usize,
}

impl Parser {
    /// Read file and create parser from it's bytes
    pub fn from_filepath(path: impl AsRef<Path>) -> std::io::Result<Parser> {
        let content = std::fs::read(path)?;

        Ok(Parser {
            bytes: content.into_boxed_slice(),
            pointer: 0,
        })
    }

    fn process_value(&mut self) -> Option<Value> {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == VALUE_UNDEFINED => Some(Value::Undefined),
            b if b == VALUE_U8 => {
                let b = &self.bytes[self.pointer];
                self.pointer += 1;

                Some(Value::U8(*b))
            }
            b if b == VALUE_I16 => {
                let b = &self.bytes[self.pointer];
                self.pointer += 1;

                let sign: i16 = if *b == 1 { 1 } else { -1 };

                let b1 = &self.bytes[self.pointer];
                self.pointer += 1;

                let b2 = &self.bytes[self.pointer];
                self.pointer += 1;

                // b1 -> 0xFF00
                // b2 -> 0x00FF
                let num = sign * ((b1 << 8) as u16 + *b2 as u16) as i16;

                Some(Value::I16(num))
            }
            b if b == VALUE_F32 => {
                let mut carrier = vec![];
                loop {
                    let b = &self.bytes[self.pointer];
                    self.pointer += 1;

                    if *b == 0 {
                        let num = String::from_utf8_lossy(&carrier)
                            .parse::<f32>()
                            .expect("Invalid f32");
                        break Some(Value::F32(num));
                    }

                    carrier.push(*b);
                }
            }
            b if b == VALUE_STRING => {
                let mut carrier = vec![];
                loop {
                    let b = &self.bytes[self.pointer];
                    self.pointer += 1;

                    if *b == 0 {
                        let s = String::from_utf8_lossy(&carrier);
                        break Some(Value::String(s.to_string()));
                    }

                    carrier.push(*b);

                    if *b == 255 {
                        let b = &self.bytes[self.pointer];
                        self.pointer += 1;

                        carrier.push(*b);
                    }
                }
            }

            _ => None,
        }
    }

    fn process_expression(&mut self) -> Option<CommandExpression> {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == EXPR_VAR => Some(CommandExpression::Var(self.process_value()?)),
            _ => {
                self.pointer -= 1; // Recover pointer
                Some(CommandExpression::Value(self.process_value()?))
            }
        }
    }

    fn step(&mut self) -> Command {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == CMD_DCLR_VAR => {
                let kind = &self.bytes[self.pointer];
                self.pointer += 1;
                let kind = match *kind as char {
                    b if b == VAR_CONST => VariableKind::Const,
                    b if b == VAR_LET => VariableKind::Let,
                    _ => panic!("Unknown variable kind"),
                };
                let name = self.process_value().expect("Invalid variable declaration");
                let value = self
                    .process_expression()
                    .expect("Invalid variable declaration");

                Command::DeclareVariable {
                    name,
                    kind,
                    value: Some(value),
                }
            }
            b if b == CMD_PUTS => Command::Puts {
                value: self.process_expression().expect("Invalid puts call"),
            },
            _ => {
                panic!("Unknown command {b}")
            }
        }
    }

    pub fn runtime(&mut self) -> Runtime {
        {
            if self.bytes.len() < 4 {
                panic!("Invalid bytecode sign");
            }

            let a = &self.bytes[0..3];
            if a != AMVM_HEADER.as_bytes() {
                panic!("Invalid bytecode sign");
            }
        }

        self.pointer = 4;

        let mut cmds = vec![];
        loop {
            if self.pointer >= self.bytes.len() {
                break;
            }

            cmds.push(self.step());
        }

        Runtime::new(cmds)
    }
}
