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

mod value;
pub use value::Aml3Value;

mod variable;
pub use variable::Aml3Variable;

use crate::Command;

pub fn from_str(source: &str) -> Result<Vec<Command>, Aml3Error> {
    let mut parser = Aml3Parser::new(Box::from(source.as_bytes()));

    Aml3Scope::visit(&mut parser, false)
}
