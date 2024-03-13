use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, BinaryConditionKind, CommandExpression, Value};

pub fn eval(
    scope: &mut AmvmScope,
    kind: &BinaryConditionKind,
    a: &CommandExpression,
    b: &CommandExpression,
) -> AmvmResult {
    let a = expr::eval(scope, a)?;
    let b = expr::eval(scope, b)?;

    match kind {
        BinaryConditionKind::GreaterThanEqual => match (a, b) {
            (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a >= b)),
            _ => todo!(),
        },
        BinaryConditionKind::NotEqual => match (a, b) {
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (Value::String(_), _) => Ok(Value::Bool(false)),
            (Value::Null, Value::Null) => Ok(Value::Bool(true)),
            (a, b) => todo!("{a:?} {b:?}"),
        },
        _ => todo!(),
    }
}
