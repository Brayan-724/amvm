use crate::{
    runtime::{commands, AmvmResult},
    tokens::{AmvmScope, Command, Value},
};

pub fn eval(scope: &mut AmvmScope, body: &[Command], use_same: bool) -> AmvmResult {
    if use_same {
        for cmd in body {
            commands::eval(scope, cmd)?;
        }
    } else {
        let mut scope = scope.create_sub(body.to_vec());

        for cmd in scope.body.clone().iter() {
            commands::eval(&mut scope, cmd)?;
        }
    }

    Ok(Value::Null)
}
