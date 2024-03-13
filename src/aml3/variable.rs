use crate::Value;

use super::{Aml3Error, Aml3Parser};

pub struct Aml3Variable;

impl Aml3Variable {
    pub fn visit(parser: &mut Aml3Parser) -> Result<Value, Aml3Error> {
        if !parser.consume_static('$') {
            return Err(parser.error_expected("$", parser.peek(0).map(|c| format!("{c:?}"))));
        }

        let name = parser
            .consume_until(' ')
            .ok_or_else(|| parser.error_expected("' '", None))?;

        Ok(Value::String(name))
    }
}
