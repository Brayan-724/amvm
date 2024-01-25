use std::error::Error;

pub mod error_msgs {
    pub const ERROR_VAR_SHOULD_BE_STRING: &'static str = "Variable names should be a string value";
    pub const ERROR_INVALID_VAR_DECL: &'static str = "Invalid variable declaration";
    pub const ERROR_UNKNOWN_VAR_KIND: &'static str = "Unknown variable kind";

    pub const ERROR_UNKNOWN_VALUE_KIND: &'static str = "Unknown value kind";
}

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
