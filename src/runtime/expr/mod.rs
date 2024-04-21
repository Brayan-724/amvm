use std::sync::Arc;

use crate::{AmvmScope, CommandExpression, Value};

use super::variable::AmvmVariable;
use super::AmvmPropagate;

mod addition;
mod binary_op;
mod cond;
mod property;
mod r#struct;
mod value;
mod var;

pub fn eval(
    scope: &mut AmvmScope,
    expr: &CommandExpression,
) -> Result<AmvmExprResult, AmvmPropagate> {
    match expr {
        CommandExpression::Prev => Ok(scope
            .context
            .read()
            .unwrap()
            .get_prev()
            .expect("No prev value")),

        CommandExpression::Value(v) => Ok(value::eval(scope, v)?.into()),
        CommandExpression::Var(v) => Ok(var::eval(scope, v)?.into()),
        CommandExpression::Property(var, property) => {
            Ok(property::eval(scope, var, property)?.into())
        }
        CommandExpression::Struct(name, body) => Ok(r#struct::eval(scope, name, body)?.into()),

        CommandExpression::Addition(a, b) => Ok(addition::eval(scope, a, b)?.into()),
        CommandExpression::Substraction(a, b) => {
            Ok(binary_op::eval(scope, binary_op::BinaryOpKind::Sub, a, b)?.into())
        }
        CommandExpression::BinaryCondition(kind, a, b) => Ok(cond::eval(scope, kind, a, b)?.into()),
    }
}

#[derive(Debug, Clone)]
pub enum AmvmExprResult {
    Value(Arc<Value>),
    Variable(AmvmVariable),
}

impl From<Value> for AmvmExprResult {
    fn from(value: Value) -> Self {
        Self::Value(Arc::new(value))
    }
}

impl From<AmvmVariable> for AmvmExprResult {
    fn from(variable: AmvmVariable) -> Self {
        Self::Variable(variable)
    }
}

impl AmvmExprResult {
    pub fn as_value(&self) -> Arc<Value> {
        match self {
            Self::Value(v) => v.clone(),
            Self::Variable(v) => v.read().clone(),
        }
    }

    pub fn as_ref(&self) -> Option<AmvmVariable> {
        match self {
            Self::Value(_) => None,
            Self::Variable(v) => Some(v.clone()),
        }
    }
}
