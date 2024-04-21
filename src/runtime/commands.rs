use crate::{AmvmScope, Command, Value};

use super::variable::AmvmVariable;
use super::{expr, AmvmResult};
use crate::runtime::scope;

mod assign_var;
mod builtin;
mod conditional;
mod r#loop;
mod puts;
mod r#struct;

pub fn eval(scope: &mut AmvmScope, cmd: &Command) -> AmvmResult {
    match cmd {
        Command::DeclareVariable { name, value, kind } => {
            let name = name.clone();
            let value = expr::eval(scope, value)?.as_value();

            scope.context.write().unwrap().variables.insert(
                name,
                AmvmVariable::new(kind.clone(), value.as_ref().clone()),
            );
            Ok(Value::Null)
        }
        Command::AssignVariable { name, value } => assign_var::eval(scope, name, value),
        Command::Puts { value } => puts::eval(scope, value),
        Command::Evaluate { expr } => Ok(expr::eval(scope, expr)?.as_value().as_ref().clone()),
        Command::Scope { body } => scope::eval(scope, body, false),
        Command::Struct { name, body } => r#struct::eval(scope, name, body),
        Command::Loop { body } => r#loop::eval(scope, body),
        Command::Conditional {
            condition,
            body,
            otherwise,
        } => conditional::eval(scope, condition, body, otherwise),
        Command::Break => Err(super::AmvmPropagate::Break),
        Command::Builtin { name, args } => builtin::eval(scope, name, args),
    }
}
