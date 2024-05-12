use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::runtime::commands::builtin;
use crate::runtime::{AmvmPropagate, AmvmVariable};
use crate::tokens::{AmvmType, CommandExpression, ValueFun, ValueObject, VariableKind};
use crate::{
    runtime::AmvmResult,
    tokens::{AmvmScope, Value},
};

use super::binary_op::BinaryOpKind;

#[derive(Clone, Copy)]
enum Kind {
    U8,
    I16,
    F32,
}

pub fn eval(scope: &mut AmvmScope, from: &CommandExpression, to: &CommandExpression) -> AmvmResult {
    let from = super::eval(scope, from)?.as_value();
    let to = super::eval(scope, to)?.as_value();
    let kind = match (&*from, &*to) {
        (Value::Null, _) | (_, Value::Null) => {
            return Err(AmvmPropagate::Err(scope.error("Null is not iterable")))
        }
        (Value::U8(_), Value::U8(_)) => Kind::U8,
        (Value::U8(_), _) => {
            return Err(AmvmPropagate::Err(
                scope.error("Range should be the same type on both sides"),
            ))
        }
        (Value::I16(_), Value::I16(_)) => Kind::I16,
        (Value::I16(_), _) => {
            return Err(AmvmPropagate::Err(
                scope.error("Range should be the same type on both sides"),
            ))
        }
        (Value::F32(_), Value::F32(_)) => Kind::F32,
        (Value::F32(_), _) => {
            return Err(AmvmPropagate::Err(
                scope.error("Range should be the same type on both sides"),
            ))
        }
        (_, _) => {
            return Err(AmvmPropagate::Err(
                scope.error("Range is only available for numbers"),
            ))
        }
    };
    let direction = match kind {
        Kind::U8 => match (&*from, &*to) {
            (Value::U8(a), Value::U8(b)) => a > b,
            _ => unreachable!(),
        },
        Kind::I16 => match (&*from, &*to) {
            (Value::I16(a), Value::I16(b)) => a > b,
            _ => unreachable!(),
        },
        Kind::F32 => match (&*from, &*to) {
            (Value::F32(a), Value::F32(b)) => a > b,
            _ => unreachable!(),
        },
    };

    let iterate = move |scope: &mut AmvmScope| -> AmvmResult {
        let to = to.clone();

        let s = scope.context.lock().unwrap();
        let Some(iterator) = s.variables.get(&String::from("self")) else {
            drop(s);
            return Err(AmvmPropagate::Err(
                scope.error("self is not defined. Please report this bug as E0001"),
            ));
        };
        let iterator = iterator.clone();
        drop(s);

        let value_ref = builtin::call(
            scope,
            ".obj.mut_access",
            &[
                iterator.clone().into(),
                Value::String(String::from("value")).into(),
            ],
        )?
        .expect(".obj.mut_access returns the reference");

        let return_value = match (kind, &*value_ref.as_value(), &*to) {
            (Kind::U8, Value::U8(value), Value::U8(to)) => {
                if value >= to {
                    Some(Value::U8(value.clone()))
                } else {
                    None
                }
            }
            (Kind::I16, Value::I16(value), Value::I16(to)) => {
                if value >= to {
                    Some(Value::I16(value.clone()))
                } else {
                    None
                }
            }
            (Kind::F32, Value::F32(value), Value::F32(to)) => {
                if value >= to {
                    Some(Value::F32(value.clone()))
                } else {
                    None
                }
            }
            _ => unreachable!(),
        };

        if let Some(val) = return_value {
            let mut obj = HashMap::new();
            obj.insert(
                String::from("done"),
                Arc::new(RwLock::new(Value::Bool(true))),
            );
            obj.insert(String::from("value"), Arc::new(RwLock::new(val)));

            return Ok(Value::Object(ValueObject::Instance(
                AmvmType::Named(Box::from("IteratorResult")),
                obj,
            )));
        }

        let value_diff = match kind {
            Kind::U8 => Value::U8(1),
            Kind::I16 => Value::I16(1),
            Kind::F32 => Value::F32(1.),
        };
        let new_value = if direction {
            super::binary_op::eval_post(
                scope,
                BinaryOpKind::Sub,
                &*value_ref.as_value(),
                &value_diff,
            )?
        } else {
            super::addition::eval_strict(scope, &*value_ref.as_value(), &value_diff)?
        };
        let variable = value_ref.as_ref();
        let mut variable = variable.write().expect("Should be mutable"); // TODO: Error propagation
        *variable = new_value.clone();

        let mut obj = HashMap::new();
        obj.insert(
            String::from("done"),
            Arc::new(RwLock::new(Value::Bool(false))),
        );
        obj.insert(
            String::from("value"),
            Arc::new(RwLock::new(new_value.clone())),
        );

        Ok(Value::Object(ValueObject::Instance(
            AmvmType::Named(Box::from("IteratorResult")),
            obj,
        )))
    };
    let iterate = ValueFun::Native(
        vec![(
            Box::from("self"),
            VariableKind::Mut,
            AmvmType::Named(Box::from("Iterator")),
        )],
        AmvmType::Named(Box::from("IteratorResult")),
        Rc::new(RefCell::new(iterate)),
    );

    let mut obj = HashMap::new();
    obj.insert(
        String::from("value"),
        Arc::new(RwLock::new((&*from).clone())),
    );
    obj.insert(
        String::from("next"),
        Arc::new(RwLock::new(Value::Fun(iterate))),
    );

    let iterator = Value::Object(ValueObject::Instance(
        AmvmType::Named(Box::from("Iterator")),
        obj,
    ));
    let iterator = AmvmVariable::new(VariableKind::Mut, iterator);

    Ok(Value::Ref(iterator))
}
