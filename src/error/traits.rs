use core::fmt;

use crate::error::{Context, ContextStack, ExpectedLength, ExpectedValid, ExpectedValue};
use crate::input::Input;

/// Convenience trait requiring both [`FromContext`] and [`FromExpected`].
pub trait Error<'i>: FromContext<'i> + FromExpected<'i> {}

impl<'i, T> Error<'i> for T where T: FromContext<'i> + FromExpected<'i> {}

/// Implemented for errors that collect contexts.
pub trait FromContext<'i> {
    /// Return `Self` with context.
    ///
    /// This method is used for adding parent contexts to errors bubbling up.
    fn from_context<C>(self, input: &'i Input, context: C) -> Self
    where
        C: Context;
}

/// Convenience trait requiring [`ExpectedValue`], [`ExpectedLength`] and
/// [`ExpectedValid`].
pub trait FromExpected<'i>:
    From<ExpectedValue<'i>> + From<ExpectedLength<'i>> + From<ExpectedValid<'i>>
{
}

impl<'i, T> FromExpected<'i> for T where
    T: From<ExpectedValue<'i>> + From<ExpectedLength<'i>> + From<ExpectedValid<'i>>
{
}

/// The required details around an error to produce a verbose report on what
/// went wrong when processing input.
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

    /// The expected value, if applicable.
    fn expected(&self) -> Option<&Input>;

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

    fn expected(&self) -> Option<&Input> {
        (**self).expected()
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).description(f)
    }

    fn context_stack(&self) -> &dyn ContextStack {
        (**self).context_stack()
    }
}
