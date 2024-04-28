use std::sync::{Arc, RwLock, RwLockWriteGuard};

use crate::{
    runtime::{AmvmError, AmvmPropagate, AmvmResult},
    tokens::{Value, VariableKind},
};

#[derive(Clone, Debug)]
pub struct Variable {
    pub kind: VariableKind,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum AmvmVariable {
    Const(Arc<Value>),
    Mut(Arc<RwLock<Value>>),
    Let(Arc<RwLock<Value>>),
    Var(Arc<RwLock<Value>>),
}

impl AmvmVariable {
    pub fn new(kind: VariableKind, value: Value) -> Self {
        match kind {
            VariableKind::Const => Self::Const(Arc::new(value)),
            VariableKind::Mut => Self::Mut(Arc::new(RwLock::new(value))),
            VariableKind::Let => Self::Let(Arc::new(RwLock::new(value))),
            VariableKind::Var => Self::Var(Arc::new(RwLock::new(value))),
        }
    }

    pub fn read(&self) -> Arc<Value> {
        match self {
            Self::Const(v) => v.clone(),
            Self::Mut(v) | Self::Let(v) | Self::Var(v) => Arc::new(v.read().unwrap().to_owned()),
        }
    }

    pub fn write(&self) -> Option<RwLockWriteGuard<'_, Value>> {
        match self {
            Self::Const(_) => None,
            Self::Mut(v) | Self::Let(v) | Self::Var(v) => Some(v.write().unwrap()),
        }
    }

    pub fn assign(&self, v: Value) -> AmvmResult {
        let variable = match self {
            Self::Const(_) => {
                return AmvmResult::Err(AmvmPropagate::Err(AmvmError::Other(
                    "Can't assign a value to a constant variable",
                )))
            }
            Self::Mut(_) => {
                return AmvmResult::Err(AmvmPropagate::Err(AmvmError::Other(
                    "Can't assign a value to a mutable variable",
                )))
            }
            Self::Let(v) => v,
            Self::Var(v) => v,
        };

        *variable.write().unwrap() = v;

        Ok(Value::Null)
    }
}
