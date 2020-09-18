use core::fmt;

use crate::error::{
    Context, ContextStack, Error, Expected, ExpectedLength, ExpectedValid, ExpectedValue,
    RetryRequirement, ToRetryRequirement,
};
use crate::input::Input;

/// `Invalid` contains no details about what happened, other than the number of
/// additional bytes required to continue processing if the error is not fatal.
///
/// This is the most performant and simplistic catch-all error, but it doesn't
/// provide any context to debug problems well.
///
/// # Example
///
/// ```
/// use dangerous::Invalid;
///
/// let error: Invalid = dangerous::input(b"").read_all(|r| {
///     r.read_u8()
/// }).unwrap_err();
///
/// assert_eq!(
///     format!("{}", error),
///     "invalid input: needs 1 byte more to continue processing",
/// );
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Invalid {
    retry_requirement: Option<RetryRequirement>,
}

impl Invalid {
    /// Create a fatal `Invalid` error.
    pub fn fatal() -> Self {
        Self {
            retry_requirement: None,
        }
    }
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid input")?;
        if let Some(retry_requirement) = self.retry_requirement {
            f.write_str(": needs ")?;
            retry_requirement.fmt(f)?;
            f.write_str(" to continue processing")?;
        }
        Ok(())
    }
}

impl ToRetryRequirement for Invalid {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

impl<'i> Error<'i> for Invalid {
    fn from_input_context<C>(self, _input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self
    }
}

impl<'i, S> From<Expected<'i, S>> for Invalid
where
    S: ContextStack,
{
    fn from(err: Expected<'i, S>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedValue<'i>> for Invalid {
    fn from(err: ExpectedValue<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedLength<'i>> for Invalid {
    fn from(err: ExpectedLength<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedValid<'i>> for Invalid {
    fn from(err: ExpectedValid<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl From<Option<RetryRequirement>> for Invalid {
    fn from(retry_requirement: Option<RetryRequirement>) -> Self {
        Self { retry_requirement }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Invalid {}
