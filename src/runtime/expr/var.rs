use crate::{
    runtime::{AmvmPropagate, AmvmVariable},
    tokens::AmvmScope,
};

pub fn eval(scope: &mut AmvmScope, var: &String) -> Result<AmvmVariable, AmvmPropagate> {
    Ok(scope.context.lock().unwrap().get_variable(var).clone())
}
