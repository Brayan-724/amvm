use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, Command, Value},
};

pub fn eval(scope: &mut AmvmScope, body: &Vec<Command>) -> AmvmResult {
    'l: loop {
        let mut scope = scope.create_sub(body.clone());

        for cmd in scope.body.clone().iter() {
            match super::eval(&mut scope, cmd) {
                Err(crate::runtime::AmvmPropagate::Break) => break 'l,
                Err(e) => return Err(e),
                _ => {}
            };
        }
    }

    Ok(Value::Null)
}
