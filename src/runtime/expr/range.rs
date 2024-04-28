use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, Value},
};

pub fn eval(_scope: &mut AmvmScope) -> AmvmResult {
    Ok(Value::U8(1))
}
