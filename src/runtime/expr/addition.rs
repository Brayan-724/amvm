use crate::{AmvmScope, AmvmTypeCasting, CommandExpression, Value};

use crate::runtime::{expr, AmvmResult};

pub fn eval(scope: &mut AmvmScope, a: &CommandExpression, b: &CommandExpression) -> AmvmResult {
    let a = expr::eval(scope, a)?;
    let b = expr::eval(scope, b)?;

    match scope.header.sum_kind {
        AmvmTypeCasting::Strict => eval_strict(scope, a, b),
        AmvmTypeCasting::TypeCastingStrict => todo!(),
        AmvmTypeCasting::TypeCastingStrictlessString => eval_cast_strictless_string(scope, a, b),
        AmvmTypeCasting::TypeCastingString => todo!(),
    }
}

fn eval_strict(_: &mut AmvmScope, a: Value, b: Value) -> AmvmResult {
    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Null),
        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a + b)),
        (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a + b)),
        (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        // TODO:
        _ => panic!("Invalid addition. \"a\" and \"b\" should be same type. "),
    }
}

fn eval_cast_strictless_string(_: &mut AmvmScope, a: Value, b: Value) -> AmvmResult {
    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Null),
        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a + b)),
        (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a + b)),
        (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        (a, b) => Ok(Value::String(format!(
            "{}{}",
            a.to_string_or_default(),
            b.to_string_or_default()
        ))),
    }
}
