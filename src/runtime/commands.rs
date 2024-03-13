use crate::{AmvmScope, Command, Value};

use super::variable::AmvmVariable;
use super::{expr, AmvmResult};
use crate::runtime::scope;

mod assign_var;
mod conditional;
mod loop_;
mod puts;

pub fn eval(scope: &mut AmvmScope, cmd: &Command) -> AmvmResult {
    match cmd {
        Command::DeclareVariable { name, value, kind } => {
            let name = name
                .as_string()
                .expect("Variable name should be string")
                .clone();
            let value = expr::eval(scope, value)?;

            scope
                .context
                .variables
                .insert(name, AmvmVariable::new(kind.clone(), value));
            Ok(Value::Null)
        }
        Command::AssignVariable { name, value } => assign_var::eval(scope, name, value),
        Command::Puts { value } => puts::eval(scope, value),
        Command::Evaluate { expr } => expr::eval(scope, expr),
        Command::Scope { body } => scope::eval(scope, body),
        Command::Loop { body } => loop_::eval(scope, body),
        Command::Conditional {
            condition,
            body,
            otherwise,
        } => conditional::eval(scope, condition, body, otherwise),
        Command::Break => Err(super::AmvmPropagate::Break),
    }
}
