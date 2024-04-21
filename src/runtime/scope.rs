use crate::{AmvmScope, Command, Value};

use super::{commands, AmvmResult};

pub fn eval(scope: &mut AmvmScope, body: &Vec<Command>, use_same: bool) -> AmvmResult {
    if use_same {
        for cmd in body.clone().iter() {
            commands::eval(scope, cmd)?;
        }
    } else {
        let mut scope = scope.create_sub(body.clone());

        for cmd in scope.body.clone().iter() {
            commands::eval(&mut scope, cmd)?;
        }
    }

    Ok(Value::Null)
}
