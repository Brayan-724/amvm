use crate::runtime::AmvmResult;
use crate::{AmvmScope, Value};

pub fn eval(_: &mut AmvmScope, v: &Value) -> AmvmResult {
    Ok(v.clone())
}
