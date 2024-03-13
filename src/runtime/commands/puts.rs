use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value};

pub fn eval(scope: &mut AmvmScope, value: &CommandExpression) -> AmvmResult {
    let value = expr::eval(scope, value)?;

    match value {
        Value::Null => print!("undefined"),
        Value::String(v) => print!("{v}"),
        Value::Bool(v) => print!("{v}"),
        Value::U8(v) => print!("{v}"),
        Value::I16(v) => print!("{v}"),
        Value::F32(v) => print!("{v}"),
    }

    Ok(Value::Null)
}
