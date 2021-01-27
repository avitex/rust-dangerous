use crate::fmt;
use crate::input::Input;

use super::{
    Context, ContextStack, Expected, ExpectedLength, ExpectedValid, ExpectedValue,
    RetryRequirement, ToRetryRequirement, WithContext,
};

/// `Invalid` contains no details around what went wrong other than a
/// [`RetryRequirement`] if the error is not fatal.
///
/// This is the most performant and simplistic catch-all **retryable** error,
/// but it doesn't provide any context to debug problems well.
///
/// See [`crate::error`] for additional documentation around the error system.
///
/// # Example
///
/// ```
/// use dangerous::{Input, Invalid};
///
/// let error: Invalid = dangerous::input(b"").read_all(|r| {
///     r.read_u8()
/// }).unwrap_err();
///
/// assert_eq!(
///     error.to_string(),
///     "invalid input: needs 1 byte more to continue processing",
/// );
/// ```
#[derive(Copy, Clone, PartialEq)]
#[must_use = "error must be handled"]
pub struct Invalid {
    retry_requirement: Option<RetryRequirement>,
}

impl Invalid {
    /// Create a fatal `Invalid` error.
    #[inline(always)]
    pub fn fatal() -> Self {
        Self {
            retry_requirement: None,
        }
    }

    /// Create a retryable `Invalid` error.
    #[inline(always)]
    pub fn retry(requirement: RetryRequirement) -> Self {
        Self {
            retry_requirement: Some(requirement),
        }
    }
}

impl fmt::Debug for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Invalid")
            .field("retry_requirement", &self.retry_requirement)
            .finish()
    }
}

impl fmt::DisplayBase for Invalid {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("invalid input")?;
        if let Some(retry_requirement) = self.retry_requirement {
            w.write_str(": needs ")?;
            fmt::DisplayBase::fmt(&retry_requirement, w)?;
            w.write_str(" to continue processing")?;
        }
        Ok(())
    }
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

impl ToRetryRequirement for Invalid {
    #[inline(always)]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

impl<'i> WithContext<'i> for Invalid {
    #[inline(always)]
    fn with_context(self, _input: impl Input<'i>, _context: impl Context) -> Self {
        self
    }
}

impl<'i, S> From<Expected<'i, S>> for Invalid
where
    S: ContextStack,
{
    #[inline(always)]
    fn from(err: Expected<'i, S>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedValue<'i>> for Invalid {
    #[inline(always)]
    fn from(err: ExpectedValue<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedLength<'i>> for Invalid {
    #[inline(always)]
    fn from(err: ExpectedLength<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl<'i> From<ExpectedValid<'i>> for Invalid {
    #[inline(always)]
    fn from(err: ExpectedValid<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

impl From<Option<RetryRequirement>> for Invalid {
    #[inline(always)]
    fn from(retry_requirement: Option<RetryRequirement>) -> Self {
        Self { retry_requirement }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Invalid {}

#[cfg(feature = "zc")]
unsafe impl zc::NoInteriorMut for Invalid {}
