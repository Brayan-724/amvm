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
            (Value::I16(a), Value::I16(b)) => Ok(Value::Bool(a >= b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::Bool(a >= b)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        BinaryKind::LessThan => match (a, b) {
            (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a < b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::Bool(a < b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::Bool(a < b)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        BinaryKind::LessThanEqual => match (a, b) {
            (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a <= b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::Bool(a <= b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::Bool(a <= b)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        BinaryKind::NotEqual => match (a, b) {
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (Value::String(_), _) => Ok(Value::Bool(false)),
            (Value::Null, Value::Null) => Ok(Value::Bool(true)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        BinaryKind::Equal => match (a, b) {
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        _ => todo!("{kind:?}"),
    }
}
