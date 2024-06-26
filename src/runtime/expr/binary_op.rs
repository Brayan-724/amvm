use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, CommandExpression, Value},
};

#[allow(unused)]
pub enum BinaryOpKind {
    Sub,
    Mult,
    Div,
}

pub fn eval(
    scope: &mut AmvmScope,
    kind: BinaryOpKind,
    a: &CommandExpression,
    b: &CommandExpression,
) -> AmvmResult {
    let binding = super::eval(scope, a)?.as_value();
    let a = binding.as_ref();
    let binding = super::eval(scope, b)?.as_value();
    let b = binding.as_ref();

    eval_post(scope, kind, a, b)
}

pub fn eval_post(_: &mut AmvmScope, kind: BinaryOpKind, a: &Value, b: &Value) -> AmvmResult {
    macro_rules! impl_ops {
        ($a:ident, $b:ident) => {
            match kind {
                BinaryOpKind::Sub => $a - $b,
                BinaryOpKind::Mult => $a * $b,
                BinaryOpKind::Div => $a / $b,
            }
        };
    }

    match (a, b) {
        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(impl_ops!(a, b))),
        (Value::I16(a), Value::I16(b)) => Ok(Value::I16(impl_ops!(a, b))),
        (Value::F32(a), Value::F32(b)) => Ok(Value::F32(impl_ops!(a, b))),

        _ => panic!("Invalid binary operation, should be the same type"),
    }
}
