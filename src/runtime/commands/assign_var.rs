use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value},
};

pub fn eval(scope: &mut AmvmScope, name: &Box<str>, value: &CommandExpression) -> AmvmResult {
    let name = name.clone();

    let value = expr::eval(scope, value)?.as_value();

    let context = scope.context.read().unwrap();
    let variable = context.get_variable(&name.to_string());
    _ = variable.assign(value.as_ref().clone())?;

    Ok(Value::Null)
}
