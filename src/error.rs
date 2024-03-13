use std::error::Error;

pub mod error_msgs {
    // Header
    pub const ERROR_INVALID_HEADER_DECL: &'static str = "Invalid header declaration";
    pub const ERROR_UNKNOWN_TYPE_CASTING_KIND: &'static str = "Unknown type casting kind";

    // Variable
    // pub const ERROR_VAR_SHOULD_BE_STRING: &'static str = "Variable names should be a string value";
    // pub const ERROR_INVALID_VAR_DECL: &'static str = "Invalid variable declaration";
    pub const ERROR_UNKNOWN_VAR_KIND: &'static str = "Unknown variable kind";

    // Value
    pub const ERROR_INVALID_VALUE_DECL: &'static str = "Invalid value declaration";
    pub const ERROR_UNKNOWN_VALUE_KIND: &'static str = "Unknown value kind";

    // Expression
    // pub const ERROR_INVALID_EXPR_DECL: &'static str = "Invalid expression declaration";

    // Command
    pub const ERROR_INVALID_CMD_DECL: &'static str = "Invalid command declaration";
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
