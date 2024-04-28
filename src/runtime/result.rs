use crate::{runtime::AmvmError, tokens::Value};

#[derive(Debug, Clone)]
pub enum AmvmPropagate {
    Return(Value),
    Break,
    Err(AmvmError),
}

impl From<AmvmError> for AmvmPropagate {
    fn from(v: AmvmError) -> Self {
        Self::Err(v)
    }
}

impl AmvmPropagate {
    /// Returns `true` if the amvm propagate is [`Break`].
    ///
    /// [`Break`]: AmvmPropagate::Break
    #[must_use]
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break)
    }

    /// Returns `true` if the amvm propagate is [`Return`].
    ///
    /// [`Return`]: AmvmPropagate::Return
    #[must_use]
    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return(..))
    }

    /// Returns `true` if the amvm propagate is [`Err`].
    ///
    /// [`Err`]: AmvmPropagate::Err
    #[must_use]
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(..))
    }

    pub fn as_err(&self) -> Option<&AmvmError> {
        if let Self::Err(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

pub type AmvmResult = Result<Value, AmvmPropagate>;
