use crate::runtime::{AmvmPropagate, AmvmVariable};
use crate::AmvmScope;

pub fn eval(scope: &mut AmvmScope, var: &String) -> Result<AmvmVariable, AmvmPropagate> {
    Ok(scope.context.read().unwrap().get_variable(var).clone())
}
