use crate::runtime::{expr, AmvmError, AmvmPropagate, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value, ValueObject};

pub fn eval(
    scope: &mut AmvmScope,
    var: &CommandExpression,
    property: &CommandExpression,
) -> AmvmResult {
    let var = expr::eval(scope, var)?.as_value();
    let var = var.as_ref();
    let property = expr::eval(scope, property)?.as_value();
    let property = property.as_ref();

    match var {
        Value::String(var) => match property {
            Value::String(prop) => match &prop as &str {
                "length" => Ok(Value::U8(var.len() as u8)),
                _ => Ok(Value::Null),
            },
            Value::U8(idx) => Ok(var
                .chars()
                .nth(*idx as usize)
                .map_or(Value::Null, |c| Value::String(String::from(c)))),
            _ => todo!(),
        },
        Value::Object(value) => match value {
            ValueObject::Native(_) => todo!("Can't get properties of native object"),
            ValueObject::Instance(_, map) => match property {
                Value::String(name) => Ok(map.get(name).cloned().unwrap_or(Value::Null)),
                _ => Err(AmvmPropagate::Err(AmvmError::Other(
                    "Objects only can be accessed by a string",
                ))),
            },
            ValueObject::PropertyMap(map) => match property {
                Value::String(name) => Ok(map.get(name).cloned().unwrap_or(Value::Null)),
                _ => Err(AmvmPropagate::Err(AmvmError::Other(
                    "Objects only can be accessed by a string",
                ))),
            },
        },
        _ => todo!(),
    }
}
