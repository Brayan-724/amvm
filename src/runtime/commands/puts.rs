use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value, ValueFun, ValueObject},
};

pub fn eval(scope: &mut AmvmScope, value: &CommandExpression) -> AmvmResult {
    let value = expr::eval(scope, value)?.as_value();
    let value = value.as_ref();

    match value {
        Value::Null => print!("undefined"),
        Value::Char(v) => print!("{v}"),
        Value::String(v) => print!("{v}"),
        Value::Bool(v) => print!("{v}"),
        Value::U8(v) => print!("{v}"),
        Value::I16(v) => print!("{v}"),
        Value::F32(v) => print!("{v}"),
        Value::Object(value) => match value {
            ValueObject::Native(ptr) => print!("[Native 0x{:02x}]", *ptr as u32),
            ValueObject::PropertyMap(map) => print!("# {map:#?}"),
            ValueObject::Instance(ty, map) => print!("{} {map:#?}", ty.flat_name()),
        },
        Value::Fun(v) => match v {
            ValueFun::Const(ref args, ret, _) | ValueFun::Mutable(ref args, ret, _) => {
                print!(
                    "[Function ({args}) {ret}]",
                    args = args
                        .iter()
                        .map(|a| format!("{name}: {ty}", name = a.0, ty = a.1.flat_name()))
                        .collect::<Vec<String>>()
                        .join(", "),
                    ret = ret.flat_name()
                )
            }
        },
    }

    Ok(Value::Null)
}
