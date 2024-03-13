use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value};

pub fn eval(
    scope: &mut AmvmScope,
    var: &CommandExpression,
    property: &CommandExpression,
) -> AmvmResult {
    let var = expr::eval(scope, var)?;
    let property = expr::eval(scope, property)?;

    match var {
        Value::String(var) => match property {
            Value::String(prop) => match &prop as &str {
                "length" => Ok(Value::U8(var.len() as u8)),
                _ => Ok(Value::Null),
            },
            Value::U8(idx) => Ok(var
                .chars()
                .nth(idx as usize)
                .map_or(Value::Null, |c| Value::String(String::from(c)))),
            _ => todo!(),
        },
        _ => todo!(),
    }
}
