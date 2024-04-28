use crate::{
    runtime::{expr, AmvmResult, AmvmVariable},
    tokens::{AmvmScope, Command, CommandExpression, Value, VariableKind},
};

#[allow(unused)]
pub fn eval(
    scope: &mut AmvmScope,
    var: &Box<str>,
    iterator: &CommandExpression,
    body: &Vec<Command>,
) -> AmvmResult {
    // TODO:
    todo!("For loop");
    let iterator = expr::eval(scope, iterator)?;

    'l: loop {
        let mut scope = scope.create_sub(body.clone());
        scope.context.write().unwrap().variables.insert(
            var.to_string(),
            AmvmVariable::new(VariableKind::Const, Value::U8(1)),
        );

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
