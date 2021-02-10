use crate::fmt;
use crate::input::{Bytes, Input, MaybeString};

use super::{Backtrace, Context, ExpectedLength, ExpectedValid, ExpectedValue, Value};
#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

/// Auto-trait for both [`WithContext`] and [`From`] for [`ExpectedValue`],
/// [`ExpectedLength`] and [`ExpectedValid`].
///
/// Also requires [`ToRetryRequirement`] if the `retry` feature is enabled.
#[cfg(feature = "retry")]
pub trait Error<'i>:
    WithContext<'i>
    + From<ExpectedValue<'i>>
    + From<ExpectedLength<'i>>
    + From<ExpectedValid<'i>>
    + ToRetryRequirement
{
}

#[cfg(feature = "retry")]
impl<'i, T> Error<'i> for T where
    T: WithContext<'i>
        + From<ExpectedValue<'i>>
        + From<ExpectedLength<'i>>
        + From<ExpectedValid<'i>>
        + ToRetryRequirement
{
}

/// Trait requiring [`WithContext`] and [`From`] for [`ExpectedValue`],
/// [`ExpectedLength`] and [`ExpectedValid`].
///
/// Also requires [`ToRetryRequirement`] if the `retry` feature is enabled.
#[cfg(not(feature = "retry"))]
pub trait Error<'i>:
    WithContext<'i> + From<ExpectedValue<'i>> + From<ExpectedLength<'i>> + From<ExpectedValid<'i>>
{
}

#[cfg(not(feature = "retry"))]
impl<'i, T> Error<'i> for T where
    T: WithContext<'i>
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

    /// Return `Self` with a parent context.
    ///
    /// This method is used for adding parent contexts to errors bubbling up.
    fn with_context(self, input: impl Input<'i>, context: impl Context) -> Self;

    /// Return `Self` with a child context attached to the last parent context
    /// added.
    ///
    /// This method is used for adding child contexts to errors bubbling up.
    fn with_child_context(self, _context: impl Context) -> Self {
        self
    }
}

/// Required details around an error to produce a verbose report on what went
/// wrong when processing input.
///
/// If you're not interested in errors of this nature and only wish to know
/// whether or not the input was correctly processed, you'll wish to use the
/// concrete type [`Invalid`] and all of the computations around verbose
/// erroring will be removed in compilation.
///
/// [`Invalid`]: crate::error::Invalid
pub trait Details<'i> {
    /// The input in its entirety that was being processed when an error
    /// occurred.
    ///
    /// The error itself will have the details and the specific section of input
    /// that caused the error. This value simply allows us to see the bigger
    /// picture given granular errors in a large amount of input.
    fn input(&self) -> MaybeString<'i>;

    /// The specific section of input that caused an error.
    fn span(&self) -> Bytes<'i>;

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
/// [`Reader::try_expect_external()`].
///
/// [`Input::into_external()`]: crate::Input::into_external()
/// [`Reader::try_expect_external()`]: crate::Reader::try_expect_external()
pub trait External<'i>: Sized {
    /// The specific section of input that caused an error.
    fn span(&self) -> Option<&'i [u8]> {
        None
    }

    /// The operation that was attempted when an error occurred.
    fn operation(&self) -> Option<&'static str> {
        None
    }

    /// The expected value.
    fn expected(&self) -> Option<&'static str> {
        None
    }

    /// Returns the requirement, if applicable, to retry processing the `Input`.
    #[cfg(feature = "retry")]
    #[cfg_attr(docsrs, doc(cfg(feature = "retry")))]
    fn retry_requirement(&self) -> Option<RetryRequirement> {
        None
    }

    /// Pushes a child backtrace to the base error generated.
    ///
    /// Push from the bottom of the trace (from the source of the error) up.
    fn push_child_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error
    }
}
