use crate::error::ParserError;
use crate::Program;
use std::path::Path;

pub(crate) const CMD_VERBOSE: bool = false;

#[derive(Debug, Clone)]
pub struct Parser {
    pub bytes: Box<[u8]>,
    pub pointer: usize,
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

    pub fn peek(&self, amount: isize) -> Option<char> {
        let (ptr, overflow) = self.pointer.overflowing_add_signed(amount);
        if overflow {
            None
        } else {
            self.bytes.get(ptr).map(|v| *v as char)
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        let r = self.bytes.get(self.pointer).map(|v| *v as char);
        self.pointer = self.pointer.saturating_add(1);

        r
    }

    pub fn next(&mut self) -> Option<char> {
        self.pointer = self.pointer.saturating_add(1);
        self.bytes.get(self.pointer).map(|v| *v as char)
    }

    /// Create a [ParserError] the context. This also use
    /// `self.pointer - prev_pointer` as the `at` argument
    pub fn error(&self, msg: impl AsRef<str>, context: String, prev_pointer: usize) -> ParserError {
        ParserError::from_msg(msg, context, self.pointer.saturating_sub(prev_pointer))
    }

    /// Create a [ParserError] with a postfix in the context referencing
    /// a possible corrupt. This also use `self.pointer - prev_pointer`
    /// as the `at` argument
    pub fn error_corrupt(
        &self,
        msg: impl AsRef<str>,
        context: impl std::fmt::Display,
        prev_pointer: usize,
    ) -> ParserError {
        ParserError::from_msg(
            msg,
            format!("{context}. Possible corrupt file or bad serialize"),
            self.pointer.saturating_sub(prev_pointer),
        )
    }

    pub fn program(&mut self) -> Result<Program, ParserError> {
        Program::visit(self)
    }
}
