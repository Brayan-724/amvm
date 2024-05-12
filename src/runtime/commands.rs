use std::rc::Rc;

use crate::tokens::AmvmMeta;
use crate::{
    runtime::{expr, scope, AmvmPropagate, AmvmResult, AmvmVariable},
    tokens::{AmvmScope, Command, Value},
};

mod assign_var;
pub mod builtin;
pub mod call;
mod conditional;
mod r#for;
mod function;
mod r#loop;
mod puts;
mod r#struct;

pub fn eval(scope: &mut AmvmScope, cmd: &Command) -> AmvmResult {
    let out = match cmd {
        Command::AssignVariable { name, value } => assign_var::eval(scope, name, value),
        Command::Break => Err(super::AmvmPropagate::Break),
        Command::Builtin { name, args } => builtin::eval(scope, name, args),
        Command::Call { name, args } => call::eval(scope, name, args),
        Command::Conditional {
            condition,
            body,
            otherwise,
        } => conditional::eval(scope, condition, body, otherwise),
        Command::DeclareVariable { name, value, kind } => {
            let name = name.clone();
            let value = expr::eval(scope, value)?.as_value_ref();

            scope.context.lock().unwrap().variables.insert(
                name.to_string(),
                AmvmVariable::new(kind.clone(), value.as_ref().clone()),
            );
            Ok(Value::Null)
        }
        Command::For {
            var,
            iterator,
            body,
        } => r#for::eval(scope, var, iterator, body),
        Command::Function {
            name,
            args,
            ret,
            body,
        } => function::eval(scope, name, args, ret, body),
        Command::Loop { body } => r#loop::eval(scope, body),
        Command::Meta { pos, code } => {
            scope.meta = Some(
                AmvmMeta {
                    file_name: scope.file_name.clone(),
                    pos: *pos,
                    code: code.clone(),
                    alternative: scope.meta.take().map(Rc::from),
                    parent: scope.meta.take().and_then(|x| x.parent.clone()),
                }
                .into(),
            );
            Ok(Value::Null)
        }
        Command::MetaFile(file_name) => {
            scope.file_name.1 = Some(file_name.clone());
            Ok(Value::Null)
        }
        Command::Push { value } => {
            let value = expr::eval(scope, value)?;
            scope.context.lock().unwrap().push_prev(value);
            Ok(Value::Null)
        }
        Command::Puts { value } => puts::eval(scope, value),
        Command::Return { value } => Err(AmvmPropagate::Return(
            expr::eval(scope, value)?.as_value().as_ref().clone(),
        )),
        Command::Scope { body } => scope::eval(scope, body, false),
        Command::Struct { name, body } => r#struct::eval(scope, name, body),
    };

    // Remove meta after each command, except for meta
    if let Command::Meta { .. } = cmd {
    } else {
        _ = scope.meta.take();
    }

    out
}
