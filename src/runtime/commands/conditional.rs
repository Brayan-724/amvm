use crate::{
    runtime::{expr, scope, AmvmResult},
    tokens::{AmvmScope, Command, CommandExpression, Value},
};

pub fn eval(
    scope: &mut AmvmScope,
    condition: &CommandExpression,
    body: &Vec<Command>,
    otherwise: &Option<Vec<Command>>,
) -> AmvmResult {
    let condition = expr::eval(scope, condition)?.as_value();
    let Value::Bool(condition) = condition.as_ref() else {
        return Err(crate::runtime::AmvmPropagate::Err(
            crate::runtime::AmvmError::Other("Condition should be boolean"),
        ));
    };

    if *condition {
        scope::eval(scope, body, false)
    } else if let Some(otherwise) = otherwise {
        scope::eval(scope, otherwise, false)
    } else {
        Ok(Value::Null)
    }
}
