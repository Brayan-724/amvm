use crate::runtime::AmvmPropagate;
use crate::{
    runtime::{expr, AmvmResult, AmvmVariable},
    tokens::{AmvmScope, Command, CommandExpression, Value, VariableKind},
};

#[allow(unused)]
pub fn eval(
    scope: &mut AmvmScope,
    var: &str,
    iterator: &CommandExpression,
    body: &Vec<Command>,
) -> AmvmResult {
    let iterator = expr::eval(scope, iterator)?.as_ref();
    let iterate = expr::property::get(
        scope,
        &*iterator.read(),
        &Value::String(String::from("next")),
    )?;
    let Some(iterate) = iterate.as_function() else {
        return Err(AmvmPropagate::Err(
            scope.error("Iterator next should be a function"),
        ));
    };

    'l: loop {
        let scope = &mut scope.create_sub(body.clone());

        let result_value = expr::property::get(
            scope,
            &*iterator.read(),
            &Value::String(String::from("value")),
        )?;

        scope.context.lock().unwrap().variables.insert(
            var.to_string(),
            AmvmVariable::new(VariableKind::Const, result_value),
        );

        for cmd in scope.body.clone().iter() {
            match super::eval(scope, cmd) {
                Err(crate::runtime::AmvmPropagate::Break) => break 'l,
                Err(e) => return Err(e),
                _ => {}
            };
        }

        let result = super::call::call(scope, iterate, &[iterator.clone()])?;

        let Value::Bool(result_done) =
            expr::property::get(scope, &result, &Value::String(String::from("done")))?
        else {
            return Err(AmvmPropagate::Err(scope.error(
                "Iterator result should have the following structure: {{ done #bool value #T }}",
            )));
        };

        if result_done {
            break;
        }
    }

    Ok(Value::Null)
}
