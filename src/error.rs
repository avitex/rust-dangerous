//! All errors supported.
use core::any::Any;
use core::fmt;
use core::num::NonZeroUsize;

use crate::error_display::ErrorDisplay;
use crate::input::Input;

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

    /// The description of what was found as opposed to what was expected.
    ///
    /// Descriptions should be simple and written in lowercase.
    ///
    /// # Errors
    ///
    /// Returns am [`fmt::Error`] if failed to write to the formatter.
    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// The expected value, if applicable.
    fn expected_value(&self) -> Option<&Input>;

    /// The description of what was expected as opposed to what was found.
    ///
    /// Descriptions should be simple and written in lowercase. They should not
    /// contain the literal value expected, that is to be left to
    /// [`ErrorDetails::expected_value()`].
    ///
    /// # Errors
    ///
    /// Returns am `fmt::Error` if failed to write to the formatter.
    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

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

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).found_description(f)
    }

    fn expected_value(&self) -> Option<&Input> {
        (**self).expected_value()
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).expected_description(f)
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
        write!(f, "{} more byte(s)", self.0)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected value error

/// An error representing a failed exact value requirement of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValue<'i> {
    pub(crate) value: &'i Input,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) operation: &'static str,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] value that was expected.
    pub fn expected(&self) -> &Input {
        self.value
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    pub(crate) fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValue<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("input not matching the expected value")
    }

    fn expected_value(&self) -> Option<&Input> {
        Some(self.value)
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("value")
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        let needed = self.value.len();
        let had = self.span().len();
        RetryRequirement::from_had_and_needed(had, needed)
    }
}

impl_error!(ExpectedValue);

///////////////////////////////////////////////////////////////////////////////
// Expected length error

/// An error representing a failed requirement for a length of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedLength<'i> {
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) operation: &'static str,
}

impl<'i> ExpectedLength<'i> {
    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use [`ErrorDetails::retry_requirement()`].
    pub fn min(&self) -> usize {
        self.min
    }

    /// The maximum length that was expected in a context, if applicable.
    ///
    /// If max has a value, this signifies the [`Input`] exceeded it in some
    /// way. An example of this would be [`Input::read_all`], where there was
    /// [`Input`] left over.
    pub fn max(&self) -> Option<usize> {
        self.max
    }

    /// Returns `true` if an exact length was expected in a context.
    pub fn is_exact(&self) -> bool {
        Some(self.min) == self.max
    }

    /// Returns `true` if `max()` has a value.
    pub fn is_fatal(&self) -> bool {
        self.max.is_some()
    }

    /// The exact length that was expected in a context, if applicable.
    ///
    /// Will return a value if `is_exact()` returns `true`.
    pub fn exact(&self) -> Option<usize> {
        if self.is_exact() {
            self.max
        } else {
            None
        }
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    pub(crate) fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedLength<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} byte(s)", self.span().len())
    }

    fn expected_value(&self) -> Option<&Input> {
        None
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (_, Some(0)) => write!(f, "no bytes"),
            (min, Some(max)) if min == max => write!(f, "{} more bytes(s)", min),
            (0, Some(max)) => write!(f, "at most {} bytes(s)", max),
            (min, None) => write!(f, "at least {} more byte(s)", min),
            (min, Some(max)) => write!(f, "at least {} and at most {} byte(s)", min, max),
        }
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let had = self.span().len();
            let needed = self.min;
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }
}

impl_error!(ExpectedLength);

///////////////////////////////////////////////////////////////////////////////
// Expected valid error

/// An error representing a failed requirement for a valid [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValid<'i> {
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) found: &'static str,
    pub(crate) expected: &'static str,
    pub(crate) operation: &'static str,
}

impl<'i> ExpectedValid<'i> {
    /// A description of what was expected in a context.
    ///
    /// Descriptions follow the conventions in
    /// [`ErrorDetails::expected_description()`].
    pub fn expected(&self) -> &'static str {
        self.expected
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    pub(crate) fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValid<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.found)
    }

    fn expected_value(&self) -> Option<&Input> {
        None
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expected)
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        None
    }
}

impl_error!(ExpectedValid);

///////////////////////////////////////////////////////////////////////////////
// All expected input errors

/// A catch-all error for all expected errors supported in this crate.
#[derive(Debug, Clone)]
pub enum Expected<'i> {
    /// An exact value was expected in a context.
    Value(ExpectedValue<'i>),
    /// A valid value was expected in a context.
    Valid(ExpectedValid<'i>),
    /// A length was expected in a context.
    Length(ExpectedLength<'i>),
}

impl<'i> Expected<'i> {
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    fn details(&self) -> &(dyn ErrorDetails<'i>) {
        match self {
            Self::Value(ref err) => err,
            Self::Valid(ref err) => err,
            Self::Length(ref err) => err,
        }
    }

    pub(crate) fn update_input(&mut self, input: &'i Input) {
        match self {
            Self::Value(ref mut err) => err.update_input(input),
            Self::Valid(ref mut err) => err.update_input(input),
            Self::Length(ref mut err) => err.update_input(input),
        }
    }
}

impl<'i> ErrorDetails<'i> for Expected<'i> {
    fn input(&self) -> &'i Input {
        self.details().input()
    }

    fn span(&self) -> &'i Input {
        self.details().span()
    }

    fn context(&self) -> &dyn Context {
        self.details().context()
    }

    fn found_value(&self) -> Option<&Input> {
        self.details().found_value()
    }

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.details().found_description(f)
    }

    fn expected_value(&self) -> Option<&Input> {
        self.details().expected_value()
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.details().expected_description(f)
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        self.details().retry_requirement()
    }
}

impl<'i> From<ExpectedValue<'i>> for Expected<'i> {
    fn from(err: ExpectedValue<'i>) -> Self {
        Self::Value(err)
    }
}

impl<'i> From<ExpectedValid<'i>> for Expected<'i> {
    fn from(err: ExpectedValid<'i>) -> Self {
        Self::Valid(err)
    }
}

impl<'i> From<ExpectedLength<'i>> for Expected<'i> {
    fn from(err: ExpectedLength<'i>) -> Self {
        Self::Length(err)
    }
}

impl_error!(Expected);

///////////////////////////////////////////////////////////////////////////////
// Basic input error

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
/// assert_eq!(format!("{}", error), "invalid input - needs 1 byte(s)")
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
            f.write_str(" - ")?;
            retry_requirement.fmt(f)?;
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
