use crate::{
    runtime::{expr, scope, AmvmPropagate, AmvmResult, AmvmVariable},
    tokens::{AmvmScope, Command, Value},
};

mod assign_var;
mod builtin;
mod call;
mod conditional;
mod r#for;
mod function;
mod r#loop;
mod puts;
mod r#struct;

pub fn eval(scope: &mut AmvmScope, cmd: &Command) -> AmvmResult {
    match cmd {
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
            let value = expr::eval(scope, value)?.as_value();

            scope.context.write().unwrap().variables.insert(
                name.to_string(),
                AmvmVariable::new(kind.clone(), value.as_ref().clone()),
            );
            Ok(Value::Null)
        }
        Command::Evaluate { expr } => Ok(expr::eval(scope, expr)?.as_value().as_ref().clone()),
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
        Command::Puts { value } => puts::eval(scope, value),
        Command::Return { value } => Err(AmvmPropagate::Return(
            expr::eval(scope, value)?.as_value().as_ref().clone(),
        )),
        Command::Scope { body } => scope::eval(scope, body, false),
        Command::Struct { name, body } => r#struct::eval(scope, name, body),
    }
}
