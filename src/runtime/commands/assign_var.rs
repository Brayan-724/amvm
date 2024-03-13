use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value, VariableKind};

pub fn eval(scope: &mut AmvmScope, name: &Value, value: &CommandExpression) -> AmvmResult {
    let name = name
        .as_string()
        .expect("Variable name should be string")
        .clone();

    {
        let variable = scope
            .context
            .variables
            .get(&name)
            .unwrap_or_else(|| panic!("{name:?} is not defined"));

        let is_mutable = variable.read().unwrap().kind == VariableKind::Let;

        if !is_mutable {
            eprintln!("ERROR: Trying to mutate inmutable variable");
            std::process::exit(1);
        }
    }

    {
        let value = expr::eval(scope, value)?;

        let variable = scope
            .context
            .variables
            .get(&name)
            .unwrap_or_else(|| panic!("{name:?} is not defined"));

        let mut variable = variable.write().unwrap();
        variable.value = value;
    }

    Ok(Value::Null)
}
