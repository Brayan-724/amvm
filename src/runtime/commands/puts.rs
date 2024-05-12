use crate::{
    runtime::{expr, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value, ValueFun, ValueObject},
};

pub fn eval(scope: &mut AmvmScope, value: &CommandExpression) -> AmvmResult {
    let value = expr::eval(scope, value)?.as_value();

    print_value(&*value);

    Ok(Value::Null)
}

fn print_value(value: &Value) {
    match value {
        Value::Null => print!("undefined"),
        Value::Bool(v) => print!("{v}"),
        Value::Char(v) => print!("{v}"),

        Value::U8(v) => print!("{v}"),
        Value::I16(v) => print!("{v}"),
        Value::F32(v) => print!("{v}"),

        Value::Ref(v) => print_value(&*v.read()),
        Value::String(v) => print!("{v}"),

        Value::Object(value) => match value {
            ValueObject::Native(ptr) => print!("[Native 0x{:02x}]", *ptr as u32),
            ValueObject::PropertyMap(map) => print!("# {map:#?}"),
            ValueObject::Instance(ty, map) => print!("{} {map:#?}", ty.flat_name()),
        },
        Value::Fun(v) => match v {
            ValueFun::Native(ref args, ret, _)
            | ValueFun::Const(ref args, ret, _)
            | ValueFun::Mutable(ref args, ret, _) => {
                print!(
                    "[Function ({args}) {ret}]",
                    args = args
                        .iter()
                        .map(|a| format!(
                            "{kind} {name}: {ty}",
                            name = a.0,
                            kind = a.1,
                            ty = a.2.flat_name()
                        ))
                        .collect::<Vec<String>>()
                        .join(", "),
                    ret = ret.flat_name()
                )
            }
        },
    }
}
