use crate::error::{error_msgs, ParserError};
use crate::{Command, CommandExpression, Runtime, Value, VariableKind};
use crate::{
    AMVM_HEADER, CMD_DCLR_VAR, CMD_PUTS, EXPR_ADD, EXPR_VAR, VALUE_F32, VALUE_I16, VALUE_STRING,
    VALUE_U8, VALUE_UNDEFINED, VAR_CONST, VAR_LET,
};
use std::path::Path;

const CMD_VERBOSE: bool = true;

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

    /// Read file and create parser from it's bytes
    pub fn from_string(path: impl AsRef<str>) -> Parser {
        Parser {
            bytes: Box::from(path.as_ref().as_bytes()),
            pointer: 0,
        }
    }

    fn process_value(&mut self) -> Result<Value, ParserError> {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == VALUE_UNDEFINED => Ok(Value::Undefined),
            b if b == VALUE_U8 => {
                let b = &self.bytes[self.pointer];
                self.pointer += 1;

                Ok(Value::U8(*b))
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

                Ok(Value::I16(num))
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
                        break Ok(Value::F32(num));
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
                        break Ok(Value::String(s.to_string()));
                    }

                    carrier.push(*b);

                    if *b == 255 {
                        let b = &self.bytes[self.pointer];
                        self.pointer += 1;

                        carrier.push(*b);
                    }
                }
            }

            b => Err(ParserError::from_msg(
                error_msgs::ERROR_UNKNOWN_VALUE_KIND,
                format!(
                    "Unrecognized byte: 0x{:02x}. Expected bytes: {:?}",
                    b as u8,
                    [
                        VALUE_UNDEFINED,
                        VALUE_U8,
                        VALUE_I16,
                        VALUE_F32,
                        VALUE_STRING
                    ]
                ),
                self.pointer - 1,
            )),
        }
    }

    fn process_expression(&mut self) -> Result<CommandExpression, ParserError> {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == EXPR_VAR => Ok(CommandExpression::Var(self.process_value()?)),
            b if b == EXPR_ADD => Ok(CommandExpression::Addition(
                self.process_expression()?.into(),
                self.process_expression()?.into(),
            )),
            _ => {
                self.pointer -= 1; // Recover pointer
                Ok(CommandExpression::Value(self.process_value()?))
            }
        }
    }

    fn step(&mut self) -> Result<Command, ParserError> {
        let b = &self.bytes[self.pointer];
        self.pointer += 1;

        match *b as char {
            b if b == CMD_DCLR_VAR => {
                let kind = &self.bytes[self.pointer];
                let kind = match *kind as char {
                    b if b == VAR_CONST => VariableKind::Const,
                    b if b == VAR_LET => VariableKind::Let,
                    b => {
                        return Err(ParserError::from_msg(
                            error_msgs::ERROR_UNKNOWN_VAR_KIND,
                            format!("{b}"),
                            self.pointer,
                        ))
                    }
                };
                self.pointer += 1;

                let name = self.process_value()?;
                let value = self.process_expression()?;

                Ok(Command::DeclareVariable { name, kind, value })
            }
            b if b == CMD_PUTS => Ok(Command::Puts {
                value: self.process_expression().expect("Invalid puts call"),
            }),
            _ => {
                panic!("Unknown command {b}")
            }
        }
    }

    pub fn runtime(&mut self) -> Result<Runtime, ParserError> {
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

            let at = self.pointer;
            let cmd = self.step()?;
            if CMD_VERBOSE {
                println!("{at}: {cmd}");
            }
            cmds.push(cmd);
        }

        Ok(Runtime::new(cmds))
    }
}
