use crate::fmt;
use crate::input::{Input, MaybeString, Span};

use super::{
    Backtrace, Context, ExpectedLength, ExpectedValid, ExpectedValue, RetryRequirement,
    ToRetryRequirement, Value,
};

/// Auto-trait for [`WithContext`], [`ToRetryRequirement`] and
/// `From<Expected(Value/Length/Valid)>`.
pub trait Error<'i>:
    WithContext<'i>
    + ToRetryRequirement
    + From<ExpectedValue<'i>>
    + From<ExpectedLength<'i>>
    + From<ExpectedValid<'i>>
{
}

impl<'i, T> Error<'i> for T where
    T: WithContext<'i>
        + ToRetryRequirement
        + From<ExpectedValue<'i>>
        + From<ExpectedLength<'i>>
        + From<ExpectedValid<'i>>
{
}

/// Implemented for errors that collect [`Context`]s.
pub trait WithContext<'i>: Sized {
    /// If `true` indicates the error does not care about any provided contexts.
    ///
    /// Defaults to `false`.
    ///
    /// This can be used for selecting a verbose error at compile time on an
    /// external parser if it is known this error will actually use the
    /// collected backtrace.
    const PASSTHROUGH: bool = false;

    /// Returns `Self` with a parent [`Input`].
    fn with_input(self, input: impl Input<'i>) -> Self;

    /// Return `Self` with a context.
    ///
    /// This method is used for adding contexts to errors bubbling up.
    fn with_context(self, context: impl Context) -> Self;
}

/// Required details around an error to produce a verbose report on what went
/// wrong when processing input.
///
/// If you're not interested in errors of this nature and only wish to know
/// whether or not the input was correctly processed, you'll wish to use the
/// concrete type [`Invalid`] and all of the computations around verbose
/// erroring will be removed in compilation.
///
/// [`Invalid`]: crate::Invalid
pub trait Details<'i> {
    /// The input in its entirety that was being processed when an error
    /// occurred.
    ///
    /// The error itself will have the details and the specific section of input
    /// that caused the error. This value simply allows us to see the bigger
    /// picture given granular errors in a large amount of input.
    fn input(&self) -> MaybeString<'i>;

    /// The expected value, if applicable.
    fn expected(&self) -> Option<Value<'_>>;

    /// The description of what went wrong while processing the input.
    ///
    /// Descriptions should be simple and written in lowercase.
    ///
    /// # Errors
    ///
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result;

    /// The walkable [`Backtrace`] to the original context around the error
    /// that occurred.
    fn backtrace(&self) -> &dyn Backtrace;
}

/// Implemented for errors that aren't a first-class citizen to `dangerous` but
/// wish to add additional information.
///
/// External errors are consumed with [`Input::into_external()`] or
/// [`Reader::try_external()`].
///
/// [`Input::into_external()`]: crate::Input::into_external()
/// [`Reader::try_external()`]: crate::Reader::try_external()
pub trait External<'i>: Sized {
    /// The specific section of input that caused an error.
    fn span(&self) -> Option<Span> {
        None
    }

    /// Returns the requirement, if applicable, to retry processing the `Input`.
    ///
    /// [`External`] errors are designed for producers of errors and won't
    /// expose themselves directly to consumers so they do not require
    /// `ToRetryRequirement` implementations but can return a
    /// [`RetryRequirement`].
    ///
    /// [`ToRetryRequirement`]: crate::ToRetryRequirement
    fn retry_requirement(&self) -> Option<RetryRequirement> {
        None
    }

    /// Pushes a child backtrace to the base error generated.
    ///
    /// Push from the bottom of the trace (from the source of the error) up.
    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error
    }
}
