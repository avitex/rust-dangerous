use core::fmt;

use crate::error::{Context, Error, RetryRequirement, ToRetryRequirement};
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
    fn with_context<C>(self, _input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self
    }
}

impl From<Option<RetryRequirement>> for Invalid {
    fn from(retry_requirement: Option<RetryRequirement>) -> Self {
        Self { retry_requirement }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Invalid {}
