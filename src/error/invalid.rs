use core::fmt;

use crate::error::{Context, Error, RetryRequirement};
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
    /// See the documentation for [`RetryRequirement`]
    pub retry_requirement: Option<RetryRequirement>,
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

impl<'i> Error<'i> for Invalid {
    fn with_context<C>(self, _input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self
    }
}

impl Default for Invalid {
    fn default() -> Self {
        Self {
            retry_requirement: None,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Invalid {}
