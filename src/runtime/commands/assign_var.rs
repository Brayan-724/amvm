use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value},
};

pub fn eval(scope: &mut AmvmScope, name: &str, value: &CommandExpression) -> AmvmResult {
    let value = expr::eval(scope, value)?.as_value();

    let context = scope.context.lock().unwrap();
    let variable = context.get_variable(&name.to_owned());
    drop(context);
    _ = variable.assign(scope, value.as_ref().clone())?;

    Ok(Value::Null)
}
