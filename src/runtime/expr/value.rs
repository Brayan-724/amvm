use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, Value},
};

pub fn eval(_: &mut AmvmScope, v: &Value) -> AmvmResult {
    Ok(v.clone())
}
