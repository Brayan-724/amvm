use std::sync::{Arc, RwLock, RwLockWriteGuard};

use crate::tokens::AmvmScope;
use crate::{
    runtime::{AmvmPropagate, AmvmResult},
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

    pub fn get_rw(&self) -> (VariableKind, Arc<RwLock<Value>>) {
        match self {
            Self::Const(_) => unimplemented!("This shouldn't be used with constant variables"),
            Self::Mut(v) => (VariableKind::Mut, Arc::clone(v)),
            Self::Let(v) => (VariableKind::Let, Arc::clone(v)),
            Self::Var(v) => (VariableKind::Var, Arc::clone(v)),
        }
    }

    pub fn from_rw(kind: VariableKind, value: Arc<RwLock<Value>>) -> Self {
        match kind {
            VariableKind::Const => unimplemented!("This shouldn't be used with constant variables"),
            VariableKind::Mut => Self::Mut(value),
            VariableKind::Let => Self::Let(value),
            VariableKind::Var => Self::Var(value),
        }
    }

    pub fn get_kind(&self) -> VariableKind {
        match self {
            Self::Const(_) => VariableKind::Const,
            Self::Let(_) => VariableKind::Let,
            Self::Mut(_) => VariableKind::Mut,
            Self::Var(_) => VariableKind::Var,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Self::Const(_) | Self::Let(_) => false,
            Self::Mut(_) | Self::Var(_) => true,
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

    pub fn assign(&self, scope: &mut AmvmScope, v: Value) -> AmvmResult {
        let variable = match self {
            Self::Const(_) => {
                return AmvmResult::Err(AmvmPropagate::Err(
                    scope.error("Can't assign a value to a constant variable"),
                ))
            }
            Self::Mut(_) => {
                return AmvmResult::Err(AmvmPropagate::Err(
                    scope.error("Can't assign a value to a mutable variable"),
                ))
            }
            Self::Let(v) => v,
            Self::Var(v) => v,
        };

        *variable.write().unwrap() = v;

        Ok(Value::Null)
    }
}
