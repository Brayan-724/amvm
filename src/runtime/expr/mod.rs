use std::sync::Arc;

use crate::{
    runtime::{AmvmPropagate, AmvmVariable},
    tokens::{AmvmScope, BinaryKind, CommandExpression, Value, VariableKind},
};

pub mod addition;
pub mod binary_op;
mod cond;
pub mod property;
mod range;
mod r#struct;
mod value;
mod var;

use binary_op::BinaryOpKind;

pub fn eval(
    scope: &mut AmvmScope,
    expr: &CommandExpression,
) -> Result<AmvmExprResult, AmvmPropagate> {
    match expr {
        CommandExpression::Binary(kind, a, b) => Ok(match kind {
            BinaryKind::Add => addition::eval(scope, a, b),
            BinaryKind::Sub => binary_op::eval(scope, BinaryOpKind::Sub, a, b),
            BinaryKind::Mult => binary_op::eval(scope, BinaryOpKind::Mult, a, b),
            _ => cond::eval(scope, kind, a, b),
        }?
        .into()),

        CommandExpression::Prev => Ok(scope
            .context
            .lock()
            .unwrap()
            .pop_prev()
            .expect("No prev value")),

        CommandExpression::Property(var, property) => {
            Ok(property::eval(scope, var, property)?.into())
        }

        CommandExpression::Range(from, to) => Ok(range::eval(scope, from, to)?.into()),
        CommandExpression::Ref(_, var) => {
            let var = eval(scope, var)?.as_ref();

            Ok(match &*var.read() {
                Value::Ref(v) => Value::Ref(v.clone()),
                _ => Value::Ref(var),
            }
            .into())
        }

        CommandExpression::Struct(name, body) => Ok(r#struct::eval(scope, name, body)?.into()),
        CommandExpression::Value(v) => Ok(value::eval(scope, v)?.into()),
        CommandExpression::Var(v) => Ok(var::eval(scope, v)?.into()),
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
        self.as_ref().read()
    }

    pub fn as_value_ref(&self) -> Arc<Value> {
        match self {
            Self::Value(v) => match &**v {
                Value::Ref(_) => Arc::clone(v),
                _ => Arc::clone(v),
            },
            Self::Variable(var) => var.read(),
        }
    }

    pub fn as_ref(&self) -> AmvmVariable {
        match self {
            Self::Value(v) => match &**v {
                Value::Ref(var) => var.clone(),
                _ => AmvmVariable::new(VariableKind::Const, Arc::as_ref(v).clone()),
            },
            Self::Variable(var) => match &*var.read() {
                Value::Ref(v) => v.clone(),
                _ => var.clone(),
            },
        }
    }

    pub fn as_var(&self) -> Option<AmvmVariable> {
        match self {
            Self::Value(_) => None,
            Self::Variable(v) => Some(v.clone()),
        }
    }
}
