use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value};

pub fn eval(scope: &mut AmvmScope, name: &String, value: &CommandExpression) -> AmvmResult {
    let name = name.clone();

    let value = expr::eval(scope, value)?.as_value();

    let context = scope.context.read().unwrap();
    let variable = context.get_variable(&name);
    _ = variable.assign(value.as_ref().clone())?;

    Ok(Value::Null)
}
