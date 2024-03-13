use std::fmt;

use super::Aml3Error;

#[derive(Debug, Clone)]
pub struct Aml3Parser {
    pub bytes: Box<[u8]>,
    pub pointer: usize,

    pub start_line: usize,
    pub line: usize,
    pub column: usize,
}

impl Aml3Parser {
    pub fn new(bytes: Box<[u8]>) -> Self {
        Self {
            bytes,
            pointer: 0,

            start_line: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.line = self.line.saturating_add(1);
        self.column = 0;
        self.start_line = self.pointer;
    }

    pub fn go_back(&mut self, amount: usize) {
        self.pointer = self.pointer.saturating_sub(amount);
        self.column = self.column.saturating_sub(amount);
    }

    #[inline(always)]
    pub fn go_forward(&mut self, amount: usize) {
        self.pointer = self.pointer.saturating_add(amount);
        self.column = self.column.saturating_add(1);
    }

    pub fn peek(&self, amount: usize) -> Option<char> {
        self.bytes
            .get(self.pointer.saturating_add(amount))
            .map(|c| *c as char)
    }

    pub fn peek_while<P>(&self, pat: P, oneline: bool) -> Option<String>
    where
        P: Fn(&char) -> bool,
    {
        let haystack = &self.bytes[self.pointer..];
        let haystack = haystack.iter().map(|c| *c as char).take_while(pat);

        let mut found = String::new();

        for c in haystack {
            if oneline && c == '\n' {
                return None;
            }

            found += &c.to_string();
        }

        if found.is_empty() {
            None
        } else {
            Some(found)
        }
    }

    pub fn peek_until(&self, pat: char) -> Option<String> {
        self.peek_while(|&c| c != pat && c != '\n', false)
    }

    pub fn peek_until_oneline(&self, pat: char) -> Option<String> {
        self.peek_while(|&c| c != pat, true)
    }

    pub fn consume_while<P>(&mut self, pat: P, oneline: bool) -> Option<String>
    where
        P: Fn(&char) -> bool,
    {
        let haystack = &self.bytes[self.pointer..];
        let haystack = haystack.iter().map(|c| *c as char).take_while(pat);

        let mut found = String::new();

        for c in haystack {
            self.pointer = self.pointer.saturating_add(1);
            self.column = self.column.saturating_add(1);

            if c == '\n' {
                self.start_line = self.pointer + 1;
                self.line += 1;
                self.column = 0;

                if oneline {
                    return None;
                }
            }

            found += &c.to_string();
        }

        if found.is_empty() {
            None
        } else {
            let c = self.bytes.get(self.pointer);
            match c.map(|c| *c as char) {
                Some('\n') => {
                    self.go_forward(1);
                    self.new_line();
                }
                Some(_) => {
                    self.go_forward(1);
                }
                _ => {}
            }

            Some(found)
        }
    }

    pub fn consume_until(&mut self, pat: char) -> Option<String> {
        self.consume_while(|&c| c != pat && c != '\n', false)
    }

    pub fn consume_until_oneline(&mut self, pat: char) -> Option<String> {
        self.consume_while(|&c| c != pat, true)
    }

    pub fn consume(&mut self) -> Option<char> {
        let r = self.bytes.get(self.pointer).map(|v| *v as char);

        if r.is_some() {
            self.pointer = self.pointer.saturating_add(1);
            self.column = self.column.saturating_add(1);
        }

        if let Some('\n') = &r {
            self.pointer = self.pointer.saturating_add(1);
            self.new_line();
        }

        r
    }

    pub fn consume_oneline(&mut self) -> Option<char> {
        let r = self.bytes.get(self.pointer).map(|v| *v as char);

        if let Some('\n') = &r {
            return None;
        }

        if r.is_some() {
            self.go_forward(1);
        }

        r
    }

    pub fn consume_static(&mut self, pat: char) -> bool {
        let a = self.bytes.get(self.pointer) == Some(&(pat as u8));

        if a {
            self.go_forward(1);
        }

        a
    }

    pub fn error(&self, description: impl fmt::Display) -> Aml3Error {
        Aml3Error {
            description: format!("{description}"),
            context: String::from_utf8_lossy(
                &self.bytes[self.start_line..(self.start_line + self.column)],
            )
            .to_string()
            .replace('\n', "\\n"),
            at: self.pointer,
            line: self.line,
            column: self.column,
        }
    }

    pub fn error_expected(&self, expected: impl fmt::Display, found: Option<String>) -> Aml3Error {
        self.error(found.map_or_else(
            || format!("Expected {expected}"),
            |f| format!("Expected {expected}. Found {f}"),
        ))
    }
}
