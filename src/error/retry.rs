use core::fmt;
use core::num::NonZeroUsize;

use crate::display::ByteCount;

/// An indicator of how many bytes are required to continue processing input.
///
/// Although the value allows you to estimate how much more input you need till
/// you can continue processing the input, it is a very granular value and may
/// result in a lot of wasted reprocessing of input if not handled correctly.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RetryRequirement(NonZeroUsize);

impl RetryRequirement {
    /// Create a new `RetryRequirement`.
    ///
    /// If the provided  value is `0`, this signifies processing can't be
    /// retried. If the provided value is greater than `0`, this signifies the
    /// amount of additional input bytes required to continue processing.
    pub fn new(value: usize) -> Option<Self> {
        NonZeroUsize::new(value).map(Self)
    }

    /// Create a retry requirement from a count of how many bytes we had and
    /// how many we needed.
    pub fn from_had_and_needed(had: usize, needed: usize) -> Option<Self> {
        Self::new(needed.saturating_sub(had))
    }

    /// Returns `true` if a provided count meets the requirement.
    pub fn met_by(self, count: usize) -> bool {
        count >= self.continue_after()
    }

    /// An indicator of how many bytes are required to continue processing input, if
    /// applicable.
    ///
    /// Although the value allows you to estimate how much more input you need till
    /// you can continue processing the input, it is a very granular value and may
    /// result in a lot of wasted reprocessing of input if not handled correctly.
    pub fn continue_after(self) -> usize {
        self.0.get()
    }

    /// Returns a `NonZeroUsize` wrapped variant of `continue_after`.
    pub fn continue_after_non_zero(self) -> NonZeroUsize {
        self.0
    }
}

impl fmt::Display for RetryRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} more", ByteCount(self.continue_after()))
    }
}

/// Implemented for errors that return a [`RetryRequirement`].
pub trait ToRetryRequirement {
    /// Returns the requirement, if applicable, to retry processing the `Input`.
    fn to_retry_requirement(&self) -> Option<RetryRequirement>;

    /// Returns `true` if [`Self::to_retry_requirement()`] will return `None`,
    /// or `false` if `Some(RetryRequirement)`.
    fn is_fatal(&self) -> bool {
        self.to_retry_requirement().is_none()
    }
}

/// Implemented for errors that can have their [`RetryRequirement`] removed.
pub trait IntoFatal: ToRetryRequirement {
    /// Returns `self` with any [`RetryRequirement`] removed.
    fn into_fatal(self) -> Self;
}

impl ToRetryRequirement for RetryRequirement {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        Some(*self)
    }
}

impl<T> ToRetryRequirement for Option<T>
where
    T: ToRetryRequirement,
{
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.as_ref()
            .and_then(ToRetryRequirement::to_retry_requirement)
    }

    fn is_fatal(&self) -> bool {
        self.is_none()
    }
}

impl<T> IntoFatal for Option<T>
where
    T: ToRetryRequirement,
{
    fn into_fatal(self) -> Self {
        None
    }
}
