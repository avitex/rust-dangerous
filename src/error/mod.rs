//! All errors supported.

mod context;
mod display;
mod expected;
mod invalid;
mod retry;

use core::fmt;

use crate::input::Input;

pub use self::context::Context;
pub use self::display::ErrorDisplay;
pub use self::expected::{Expected, ExpectedLength, ExpectedValid, ExpectedValue};
pub use self::invalid::Invalid;
pub use self::retry::{RetryRequirement, ToRetryRequirement};

#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) use self::context::ContextNode;
pub(crate) use self::context::{ExpectedContext, OperationContext};
pub(crate) use self::display::fmt_debug_error;
pub(crate) use self::expected::Value;

/// Core error that collects contexts.
pub trait Error<'i>: ToRetryRequirement {
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
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
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
}
