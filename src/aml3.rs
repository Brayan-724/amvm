mod command;
pub use command::Aml3Command;

mod error;
pub use error::Aml3Error;

mod expr;
pub use expr::Aml3Expr;

mod parser;
pub use parser::Aml3Parser;

mod scope;
pub use scope::Aml3Scope;

mod r#struct;
pub use r#struct::Aml3Struct;

mod r#type;
pub use r#type::Aml3Type;

mod value;
pub use value::Aml3Value;

mod variable;
pub use variable::Aml3Variable;

use crate::parser::Parser;
use crate::tokens::Command;

pub fn from_str(source: &str) -> Result<Vec<Command>, String> {
    let parser = Parser::new(source, &false);
    let (_, c) = Aml3Scope::visit(parser, false).map_err(Parser::flat_errors)?;

    Ok(c)
}
