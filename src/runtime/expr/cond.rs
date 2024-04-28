use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, BinaryKind, CommandExpression, Value},
};

pub fn eval(
    scope: &mut AmvmScope,
    kind: &BinaryKind,
    a: &CommandExpression,
    b: &CommandExpression,
) -> AmvmResult {
    let binding = expr::eval(scope, a)?.as_value();
    let a = binding.as_ref();
    let binding = expr::eval(scope, b)?.as_value();
    let b = binding.as_ref();

    match kind {
        BinaryKind::GreaterThanEqual => match (a, b) {
            (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a >= b)),
            _ => todo!(),
        },
        BinaryKind::NotEqual => match (a, b) {
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (Value::String(_), _) => Ok(Value::Bool(false)),
            (Value::Null, Value::Null) => Ok(Value::Bool(true)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        BinaryKind::Equal => match (a, b) {
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        _ => todo!(),
    }
}
