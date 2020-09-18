//! Error support
//!
//! - If you want the fastest error which has no debugging information,
//!   [`Invalid`] has you covered.
//! - If you want an error that is still designed to be fast, but also includes
//!   debugging information, [`Expected`] will meet your uh, expectations...
//! - If you require more verbosity, consider creating custom [`Context`]s
//!   before jumping to custom errors. If you do require a custom error,
//!   implementing it is easy enough. Just implement [`Error`] and [`From`] for
//!   [`ExpectedValue`], [`ExpectedValid`] and [`ExpectedLength`] and you'll be
//!   on your merry way. Additionally implement [`ErrorDisplay`] to support
//!   lovely error printing and [`ToRetryRequirement`] for streaming protocols.
//!
//! Most of what `dangerous` supports out of the box is good to go. If you need
//! to stretch out performance more, or provide additional functionality on what
//! is provided, the error system should be flexible for those requirements. If
//! it's not, consider opening an issue.

mod context;
mod display;
mod expected;
mod invalid;
mod retry;

use core::fmt;

use crate::input::Input;

#[cfg(feature = "full-context")]
pub use self::context::FullContextStack;
pub use self::context::{
    Context, ContextStack, ContextStackBuilder, ContextStackWalker, ExpectedContext,
    RootContextStack,
};
pub use self::display::ErrorDisplay;
pub use self::expected::{Expected, ExpectedLength, ExpectedValid, ExpectedValue, FromExpected};
pub use self::invalid::Invalid;
pub use self::retry::{RetryRequirement, ToRetryRequirement};

pub(crate) use self::context::OperationContext;
pub(crate) use self::expected::Value;

/// Core error that collects contexts.
pub trait Error<'i> {
    /// Return `Self` with context.
    ///
    /// This method is used for adding parent contexts to errors bubbling up.
    /// How child and parent contexts are handled are upstream concerns.
    fn from_input_context<C>(self, input: &'i Input, context: C) -> Self
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

    /// The walkable [`ContextStack`] to the original context around the error
    /// that occured.
    fn context_stack(&self) -> &dyn ContextStack;
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

    fn found_value(&self) -> Option<&Input> {
        (**self).found_value()
    }

    fn expected_value(&self) -> Option<&Input> {
        (**self).expected_value()
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).description(f)
    }

    fn context_stack(&self) -> &dyn ContextStack {
        (**self).context_stack()
    }
}
