use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::runtime::AmvmError;
use crate::CompileResult;
use crate::{
    runtime::Context,
    tokens::{AmvmHeader, Command},
    Compilable,
};

#[derive(Debug, Clone)]
pub struct AmvmMeta {
    pub file_name: (Option<Box<str>>, Option<Box<str>>),
    pub pos: (u16, u16),
    pub code: Box<str>,
    pub alternative: Option<Rc<AmvmMeta>>,
    pub parent: Option<Rc<AmvmMeta>>,
}

impl AmvmMeta {
    pub fn display(&self, is_original: bool, is_debug: bool) -> String {
        use fmt::Write;
        let mut f = String::new();

        let debug_char = if is_debug { "> " } else { "" };

        let file_name = if is_original {
            self.file_name.0.as_ref().map(|x| x.as_ref())
        } else {
            self.file_name.1.as_ref().map(|x| x.as_ref())
        }
        .unwrap_or("<anonymous>");

        let position = format!("{}:{}", self.pos.0, self.pos.1);
        _ = writeln!(f, "\x1b[1;34m{debug_char}--> {file_name}:{position}\x1b[0m");

        let line = self.pos.0.to_string();
        let line_pad = " ".repeat(line.len());
        let col = self.pos.1;

        let code_len = self.code.len();
        let (code, start) = if code_len >= 40 {
            let dots = "\x1b[1;34m...\x1b[0m";
            let start = col.saturating_sub(20) as usize;
            let end = (col.saturating_add(40 - start as u16) as usize).min(code_len);
            let start_dots = if start > 3 { dots } else { "" };
            let end_dots = if end != 0 { dots } else { "" };
            let code = format!("{}{}{}", start_dots, &self.code[start..end], end_dots);

            (
                code,
                if start != 0 {
                    start.saturating_sub(3)
                } else {
                    start
                },
            )
        } else {
            (self.code.to_string(), 0)
        };
        let code_line = code;
        let cursor_pad = " ".repeat(self.pos.1 as usize - start);

        _ = writeln!(f, "\x1b[1;34m{debug_char}  {line} | \x1b[0m{code_line}");
        _ = writeln!(f, "\x1b[1;34m{debug_char}  {line_pad} | {cursor_pad}^");

        f
    }
}

#[derive(Debug, Clone)]
pub struct AmvmScope {
    pub file_name: (Option<Box<str>>, Option<Box<str>>),
    pub meta: Option<Rc<AmvmMeta>>,
    pub backtrace: Option<Rc<AmvmMeta>>,
    pub header: Rc<AmvmHeader>,
    pub body: Rc<Vec<Command>>,
    pub context: Arc<Mutex<Context>>,
}

impl AmvmScope {
    pub fn new(
        file_name: Box<str>,
        header: &Rc<AmvmHeader>,
        body: Vec<Command>,
        upper: Option<Arc<Mutex<Context>>>,
    ) -> Self {
        let ctx = upper.map_or_else(Context::new, Context::create_sub);
        Self {
            file_name: (Some(file_name), None),
            meta: None,
            backtrace: None,
            header: Rc::clone(header),
            body: Rc::new(body),
            context: Arc::new(Mutex::new(ctx)),
        }
    }

    pub fn create_sub(&self, body: Vec<Command>) -> Self {
        Self {
            file_name: self.file_name.clone(),
            meta: None,
            backtrace: self
                .meta
                .clone()
                .map(|meta| {
                    let mut meta = (&*meta).clone();
                    meta.parent = self.backtrace.clone().map(Rc::from);
                    Rc::new(meta)
                })
                .or(self.backtrace.clone()),
            header: Rc::clone(&self.header),
            body: Rc::new(body),
            context: Arc::new(Mutex::new(Context::create_sub(self.context.clone()))),
        }
    }

    pub fn full_backtrace(&self) -> Vec<Rc<AmvmMeta>> {
        let mut out = vec![];
        if let Some(meta) = &self.meta {
            out.push(meta.clone());
        }

        if let Some(meta) = &self.backtrace {
            out.push(meta.clone());
            let mut parent = meta.parent.as_ref();

            while let Some(meta) = parent {
                out.push(meta.clone());
                parent = meta.parent.as_ref();
            }
        }

        out
    }

    pub fn error(&mut self, ctx: &'static str) -> AmvmError {
        AmvmError::Other(self.full_backtrace(), ctx)
    }
}

impl Compilable for AmvmScope {
    fn compile_bytecode(&self, buffer: String) -> CompileResult {
        self.body.compile_bytecode(buffer)
    }
}

impl fmt::Display for AmvmScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{body:?}", body = self.body))
    }
}
