use core::fmt;

use crate::input::Input;

use super::{
    Context, ContextStack, Expected, ExpectedLength, ExpectedValid, ExpectedValue, FromContext,
    IntoFatal, RetryRequirement, ToRetryRequirement,
};

/// `Invalid` contains no details around what went wrong other than a
/// [`RetryRequirement`] if the error is not fatal.
///
/// This is the most performant and simplistic catch-all error, but it doesn't
/// provide any context to debug problems well.
///
/// See [`crate::error`] for additional documentation around the error system.
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
///     error.to_string(),
///     "invalid input: needs 1 byte more to continue processing",
/// );
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
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
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    #[inline(always)]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

impl IntoFatal for Invalid {
    #[inline(always)]
    fn into_fatal(self) -> Self {
        Self::fatal()
    }
}

impl<'i> FromContext<'i> for Invalid {
    #[inline(always)]
    fn from_context<C>(self, _input: Input<'i>, _context: C) -> Self
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
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Invalid {}
