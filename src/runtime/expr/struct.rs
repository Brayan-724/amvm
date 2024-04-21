use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, AmvmType, CommandExpression, Value, ValueObject};

pub fn eval(
    scope: &mut AmvmScope,
    ty: &AmvmType,
    body: &Vec<(Box<str>, CommandExpression)>,
) -> AmvmResult {
    let mut body_evaluated = HashMap::new();

    for (prop_name, prop_value) in body {
        let prop_name = prop_name.to_string();
        let prop_value = expr::eval(scope, prop_value)?.as_value();
        let prop_value =
            Arc::into_inner(prop_value).expect("Can't get inner value on struct expression");

        body_evaluated.insert(prop_name, prop_value);
    }

    Ok(Value::Object(ValueObject::Instance(
        ty.clone(),
        body_evaluated,
    )))
}
