use crate::runtime::{expr, scope, AmvmResult};
use crate::{AmvmScope, Command, CommandExpression, Value};

pub fn eval(
    scope: &mut AmvmScope,
    condition: &CommandExpression,
    body: &Vec<Command>,
    otherwise: &Option<Vec<Command>>,
) -> AmvmResult {
    let condition = expr::eval(scope, condition)?;
    let Value::Bool(condition) = condition else {
        return Ok(Value::Null);
    };

    if condition {
        scope::eval(scope, body)
    } else if let Some(otherwise) = otherwise {
        scope::eval(scope, otherwise)
    } else {
        Ok(Value::Null)
    }
}
