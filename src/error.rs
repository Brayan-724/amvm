use std::error::Error;

#[derive(Debug, Clone)]
pub struct ParserError {
    description: Box<str>,
    context: String,
    at: usize,
}

impl ParserError {
    pub fn from_msg(msg: impl AsRef<str>, context: String, at: usize) -> Self {
        Self {
            description: Box::from(msg.as_ref()),
            context,
            at,
        }
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "error parser({at}) {desc}.\nContext: {context}",
            at = self.at,
            desc = &self.description,
            context = self.context
        ))
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        &self.description
    }
}
