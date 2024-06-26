use crate::{
    runtime::{AmvmResult, AmvmVariable},
    tokens::{AmvmScope, AmvmType, Command, Value, ValueFun, VariableKind},
};

pub fn eval(
    scope: &mut AmvmScope,
    name: &str,
    args: &Vec<(Box<str>, VariableKind, AmvmType)>,
    ret: &AmvmType,
    body: &Vec<Command>,
) -> AmvmResult {
    let value = ValueFun::Const(args.clone(), ret.clone(), body.clone());
    let value = Value::Fun(value);

    let name = name.to_string();
    scope
        .context
        .lock()
        .unwrap()
        .variables
        .insert(name, AmvmVariable::new(VariableKind::Const, value));

    Ok(Value::Null)
}
