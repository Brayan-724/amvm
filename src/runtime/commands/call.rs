use crate::{
    runtime::{expr, scope, AmvmError, AmvmPropagate, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value, ValueFun},
};

pub fn eval(
    scope: &mut AmvmScope,
    name: &CommandExpression,
    args: &Vec<CommandExpression>,
) -> AmvmResult {
    let name = expr::eval(scope, name)?;
    let name = name.as_value();
    let Some(name) = name.as_string() else {
        return Err(AmvmPropagate::Err(AmvmError::Other(
            "Function name should be string",
        )));
    };

    let fun = scope
        .context
        .read()
        .unwrap()
        .get_variable(&name.to_string());

    let fun = fun.read();
    let fun = fun.as_ref();
    let fun = match fun {
        Value::Fun(ref fun) => fun,
        _ => {
            return Err(AmvmPropagate::Err(AmvmError::Other(
                "Calling to non-function",
            )))
        }
    };

    let (named_args, body) = match fun {
        ValueFun::Const(a, _, b) => (a, b),
        ValueFun::Mutable(a, _, b) => (a, b),
    };

    let inner = &mut scope.create_sub(body.to_vec());

    for (value, (name, _)) in args.iter().zip(named_args) {
        let name = name.to_string();
        let value = expr::eval(scope, value)?.as_ref();

        inner.context.write().unwrap().variables.insert(name, value);
    }

    let value = match scope::eval(inner, body, true) {
        Ok(v) => v,
        Err(AmvmPropagate::Return(v)) => v,
        Err(e) => return Err(e),
    };

    scope.context.read().unwrap().set_prev_value(value);

    Ok(Value::Null)
}
