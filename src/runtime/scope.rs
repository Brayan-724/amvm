use crate::{AmvmScope, Command, Value};

use super::{commands, AmvmResult};

pub fn eval(scope: &mut AmvmScope, body: &Vec<Command>) -> AmvmResult {
    let mut scope = scope.create_sub(body.clone());

    for cmd in scope.body.clone().iter() {
        commands::eval(&mut scope, cmd)?;
    }

    Ok(Value::Null)
}
