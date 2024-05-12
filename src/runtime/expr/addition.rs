use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, AmvmTypeCasting, CommandExpression, Value},
};

pub fn eval(scope: &mut AmvmScope, a: &CommandExpression, b: &CommandExpression) -> AmvmResult {
    let binding = expr::eval(scope, a)?.as_value();
    let a = binding.as_ref();
    let binding = expr::eval(scope, b)?.as_value();
    let b = binding.as_ref();

    match scope.header.sum_kind {
        AmvmTypeCasting::Strict => eval_strict(scope, a, b),
        AmvmTypeCasting::TypeCastingStrict => todo!(),
        AmvmTypeCasting::TypeCastingStrictlessString => eval_cast_strictless_string(scope, a, b),
        AmvmTypeCasting::TypeCastingString => todo!(),
    }
}

pub fn eval_strict(scope: &mut AmvmScope, a: &Value, b: &Value) -> AmvmResult {
    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Null),
        (Value::U8(a), Value::U8(b)) => {
            Ok(Value::U8(a.checked_add(*b).ok_or_else(|| {
                scope.error("Attempt to add with overflow")
            })?))
        }
        (Value::I16(a), Value::I16(b)) => {
            Ok(Value::I16(a.checked_add(*b).ok_or_else(|| {
                scope.error("Attempt to add with overflow")
            })?))
        }
        (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        // TODO:
        _ => panic!("Invalid addition. \"a\" and \"b\" should be same type. "),
    }
}

fn eval_cast_strictless_string(scope: &mut AmvmScope, a: &Value, b: &Value) -> AmvmResult {
    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Null),
        (Value::U8(_), Value::U8(_)) => eval_strict(scope, a, b),
        (Value::I16(_), Value::I16(_)) => eval_strict(scope, a, b),
        (Value::F32(_), Value::F32(_)) => eval_strict(scope, a, b),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        (a, b) => Ok(Value::String(format!(
            "{}{}",
            a.to_string_or_default(),
            b.to_string_or_default()
        ))),
    }
}
