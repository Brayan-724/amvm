use crate::runtime::AmvmResult;
use crate::{AmvmScope, Value};

pub fn eval(scope: &mut AmvmScope, v: &Value) -> AmvmResult {
    let var = v.as_string().expect("Variable name should be string");
    Ok(scope
        .context
        .get_variable(var)
        .read()
        .unwrap()
        .clone()
        .value)
}
