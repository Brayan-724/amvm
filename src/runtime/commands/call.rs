use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::AmvmVariable;
use crate::tokens::{Command, VariableKind};
use crate::{
    runtime::{expr, scope, AmvmPropagate, AmvmResult},
    tokens::{AmvmScope, CommandExpression, Value, ValueFun},
};

pub fn eval(
    scope: &mut AmvmScope,
    name: &CommandExpression,
    args: &[CommandExpression],
) -> AmvmResult {
    let name = expr::eval(scope, name)?;
    let name = name.as_value();
    let Some(fun) = name.as_function() else {
        return Err(AmvmPropagate::Err(scope.error("Calling to a non-function")));
    };

    let mut args_evaluated = Vec::with_capacity(args.len());

    for arg in args {
        args_evaluated.push(expr::eval(scope, arg)?.as_ref());
    }

    let value = call(scope, fun, &args_evaluated)?;

    scope.context.lock().unwrap().push_prev_value(value);

    Ok(Value::Null)
}

enum Either<'a> {
    Native(Rc<RefCell<dyn FnMut(&mut AmvmScope) -> AmvmResult>>),
    Body(&'a Vec<Command>),
}

pub fn call(scope: &mut AmvmScope, fun: &ValueFun, args: &[AmvmVariable]) -> AmvmResult {
    let (named_args, body, mut inner) = match fun {
        ValueFun::Native(a, _, b) => (a, Either::Native(b.clone()), scope.create_sub(vec![])),
        ValueFun::Const(a, _, b) | ValueFun::Mutable(a, _, b) => {
            (a, Either::Body(b), scope.create_sub(b.to_vec()))
        }
    };

    let inner = &mut inner;

    for (value, (name, arg_kind, _)) in args.iter().zip(named_args) {
        let name = name.to_string();
        let value_kind = value.get_kind();

        assert!(
            *arg_kind <= value_kind,
            "Cannot cast {value_kind} to {arg_kind}"
        );

        let arg = if *arg_kind == VariableKind::Const {
            AmvmVariable::new(VariableKind::Const, (*value.read()).clone())
        } else {
            AmvmVariable::from_rw(*arg_kind, value.get_rw().1)
        };

        inner.context.lock().unwrap().variables.insert(name, arg);
    }

    match body {
        Either::Body(body) => {
            let value = match scope::eval(inner, body, true) {
                Ok(v) => v,
                Err(AmvmPropagate::Return(v)) => v,
                Err(e) => return Err(e),
            };

            Ok(value)
        }
        Either::Native(ref fun) => (fun.borrow_mut())(inner),
    }
}
