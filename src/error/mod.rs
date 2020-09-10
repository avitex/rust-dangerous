//! All errors supported.

mod display;
mod expected;
mod invalid;
#[cfg(any(feature = "std", feature = "alloc"))]
mod verbose;

use core::any::Any;
use core::fmt;
use core::num::NonZeroUsize;

use crate::input::Input;
use crate::utils::ByteCount;

pub use self::display::ErrorDisplay;
pub use self::expected::{Expected, ExpectedLength, ExpectedValid, ExpectedValue};
pub use self::invalid::Invalid;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use self::verbose::VerboseError;

/// Core error that collects contexts.
pub trait Error<'i> {
    /// Return `Self` with context.
    ///
    /// This method is used for adding parent contexts to errors bubbling up.
    /// How child and parent contexts are handled are upstream concerns.
    fn with_context<C>(self, input: &'i Input, context: C) -> Self
    where
        C: Context;
}

/// The errors details around an error produced while attempting to process
/// input providing the required properties to produce a verbose report on what
/// happened.
///
/// If you're not interested in errors of this nature and only wish to know
/// whether or not the input was correctly processed, you'll wish to use the
/// concrete type `Invalid` and all of the computations around verbose erroring
/// will be removed in compilation.
pub trait ErrorDetails<'i> {
    /// The input in its entirety that was being processed when an error
    /// occured.
    ///
    /// The error itself will have the details and the specific section of input
    /// that caused the error. This value simply allows us to see the bigger
    /// picture given granular errors in a large amount of input.
    fn input(&self) -> &'i Input;

    /// The specific section of input that caused an error.
    fn span(&self) -> &'i Input;

    /// The context around the error.
    fn context(&self) -> &dyn Context;

    /// The unexpected value, if applicable, that was found.
    fn found_value(&self) -> Option<&Input>;

    /// The expected value, if applicable.
    fn expected_value(&self) -> Option<&Input>;

    /// The description of what went wrong while processing the input.
    ///
    /// Descriptions should be simple and written in lowercase.
    ///
    /// # Errors
    ///
    /// Returns am [`fmt::Error`] if failed to write to the formatter.
    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Returns the requirement, if applicable, to retry processing the `Input`.
    fn retry_requirement(&self) -> Option<RetryRequirement>;
}

impl<'i, T> ErrorDetails<'i> for &T
where
    T: ErrorDetails<'i>,
{
    fn input(&self) -> &'i Input {
        (**self).input()
    }

    fn span(&self) -> &'i Input {
        (**self).span()
    }

    fn context(&self) -> &dyn Context {
        (**self).context()
    }

    fn found_value(&self) -> Option<&Input> {
        (**self).found_value()
    }

    fn expected_value(&self) -> Option<&Input> {
        (**self).expected_value()
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).description(f)
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        (**self).retry_requirement()
    }
}

/// The context surrounding an error.
pub trait Context: Any {
    /// The operation that was attempted when an error occured.
    ///
    /// It should described in a simple manner what is trying to be achieved and
    /// make sense in the following sentence if you were to substitute it:
    ///
    /// ```text
    /// Something failed while attempting to <operation> from the input.
    /// ```
    fn operation(&self) -> &'static str;

    /// The more granular context of where the error occured.
    ///
    /// # Example
    ///
    /// Say we attempted to process a UTF-8 string from the input via
    /// [`Input::to_dangerous_str()`] within a parent operation described
    /// `decode name`. The final context produced would be that of around
    /// `decode name`. The `child` context would be that of
    /// [`Input::to_dangerous_str()`].
    ///
    /// This would allow us to walk the contexts, so we can present the
    /// following information for use in debugging:
    ///
    /// ```text
    /// UTF-8 error occured while attempting to decode name from the input.
    ///
    /// context backtrace:
    /// 1. `decode name`: expected valid name
    /// 2. `decode utf-8 code point`: invalid utf-8 code point encounted
    /// ```
    fn child(&self) -> Option<&dyn Context>;

    /// The number of child contexts consolidated into `self`.
    ///
    /// Any context returned from `child` is the next deeper than those that
    /// were consolidated.
    fn consolidated(&self) -> usize;
}

impl Context for &'static str {
    fn operation(&self) -> &'static str {
        self
    }

    fn child(&self) -> Option<&dyn Context> {
        None
    }

    fn consolidated(&self) -> usize {
        0
    }
}

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

    /// Returns `true` if a provided count mets the requirement.
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} more", ByteCount(self.continue_after()))
    }
}
