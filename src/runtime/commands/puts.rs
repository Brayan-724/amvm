use crate::runtime::{expr, AmvmResult};
use crate::{AmvmScope, CommandExpression, Value, ValueObject};

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
    }

    Ok(Value::Null)
}
