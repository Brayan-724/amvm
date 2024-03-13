use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Value, VariableKind};

#[derive(Clone, Debug)]
pub struct Variable {
    pub kind: VariableKind,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct AmvmVariable {
    inner: Arc<RwLock<Variable>>,
}

impl AmvmVariable {
    pub fn new(kind: VariableKind, value: Value) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Variable { kind, value })),
        }
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, Variable>> {
        self.inner.read()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, Variable>> {
        self.inner.write()
    }
}
