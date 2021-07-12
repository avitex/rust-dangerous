mod length;
mod valid;
mod value;

pub use self::length::ExpectedLength;
pub use self::valid::ExpectedValid;
pub use self::value::ExpectedValue;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::display::ErrorDisplay;
use crate::error::{
    Backtrace, BacktraceBuilder, Context, Details, RetryRequirement, ToRetryRequirement, Value,
    WithContext,
};
use crate::fmt;
use crate::input::{Input, MaybeString};

#[cfg(feature = "full-backtrace")]
type ExpectedBacktrace = crate::error::FullBacktrace;
#[cfg(not(feature = "full-backtrace"))]
type ExpectedBacktrace = crate::error::RootBacktrace;

/// An error that [`Details`] what went wrong while reading and may be retried.
///
/// - Enable the `full-backtrace` feature (enabled by default), to collect of
///   all contexts with [`Expected`].
/// - It is generally recommended for better performance to box `Expected` if
///   the structures being returned from parsing are smaller than or equal to
///   `~128 bytes`. This is because the `Expected` structure is `192 - 216
///   bytes` large on 64 bit systems and successful parses may be hindered by
///   the time to move the `Result<T, Expected>` value. By boxing `Expected` the
///   size becomes only `8 bytes`. When in doubt, write a benchmark.
///
/// See [`crate::error`] for additional documentation around the error system.
#[must_use = "error must be handled"]
pub struct Expected<'i, S = ExpectedBacktrace> {
    input: MaybeString<'i>,
    trace: S,
    kind: ExpectedKind<'i>,
}

enum ExpectedKind<'i> {
    /// An exact value was expected in a context.
    Value(ExpectedValue<'i>),
    /// A valid value was expected in a context.
    Valid(ExpectedValid<'i>),
    /// A length was expected in a context.
    Length(ExpectedLength<'i>),
}

impl<'i, S> Expected<'i, S>
where
    S: Backtrace,
{
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<'_, Self> {
        ErrorDisplay::new(self)
    }
}

impl<'i, S> Expected<'i, S>
where
    S: BacktraceBuilder,
{
    #[inline(always)]
    fn add_input(&mut self, input: impl Input<'i>) {
        if self.input.span().is_within(input.span()) {
            self.input = input.into_maybe_string();
        }
    }

    #[inline(always)]
    fn add_context(&mut self, context: impl Context) {
        self.trace.push(context);
    }

    fn from_kind(kind: ExpectedKind<'i>) -> Self {
        let (input, context) = match &kind {
            ExpectedKind::Valid(err) => (err.input(), err.context()),
            ExpectedKind::Value(err) => (err.input(), err.context()),
            ExpectedKind::Length(err) => (err.input(), err.context()),
        };
        Self {
            kind,
            input,
            trace: S::from_root(context),
        }
    }
}

impl<'i, S> Details<'i> for Expected<'i, S>
where
    S: Backtrace,
{
    fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }

    fn expected(&self) -> Option<Value<'i>> {
        match &self.kind {
            ExpectedKind::Value(err) => Some(err.expected()),
            ExpectedKind::Valid(_) | ExpectedKind::Length(_) => None,
        }
    }

    fn description(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        match &self.kind {
            ExpectedKind::Value(err) => fmt::DisplayBase::fmt(err, f),
            ExpectedKind::Valid(err) => fmt::DisplayBase::fmt(err, f),
            ExpectedKind::Length(err) => fmt::DisplayBase::fmt(err, f),
        }
    }

    fn backtrace(&self) -> &dyn Backtrace {
        &self.trace
    }
}

impl<'i, S> ToRetryRequirement for Expected<'i, S> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        match &self.kind {
            ExpectedKind::Value(err) => err.to_retry_requirement(),
            ExpectedKind::Valid(err) => err.to_retry_requirement(),
            ExpectedKind::Length(err) => err.to_retry_requirement(),
        }
    }

    fn is_fatal(&self) -> bool {
        match &self.kind {
            ExpectedKind::Value(err) => err.is_fatal(),
            ExpectedKind::Valid(err) => err.is_fatal(),
            ExpectedKind::Length(err) => err.is_fatal(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> ToRetryRequirement for Box<Expected<'i, S>> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        (**self).to_retry_requirement()
    }

    fn is_fatal(&self) -> bool {
        (**self).is_fatal()
    }
}

impl<'i, S> WithContext<'i> for Expected<'i, S>
where
    S: BacktraceBuilder,
{
    const PASSTHROUGH: bool = S::PASSTHROUGH;

    fn with_input(mut self, input: impl Input<'i>) -> Self {
        self.add_input(input);
        self
    }

    fn with_context(mut self, context: impl Context) -> Self {
        self.add_context(context);
        self
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> WithContext<'i> for Box<Expected<'i, S>>
where
    S: BacktraceBuilder,
{
    const PASSTHROUGH: bool = S::PASSTHROUGH;

    fn with_input(mut self, input: impl Input<'i>) -> Self {
        self.add_input(input);
        self
    }

    fn with_context(mut self, context: impl Context) -> Self {
        self.add_context(context);
        self
    }
}

impl<'i, S> fmt::Debug for Expected<'i, S>
where
    S: Backtrace,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorDisplay::from_formatter(self, f).banner(true).fmt(f)
    }
}

impl<'i, S> fmt::Display for Expected<'i, S>
where
    S: Backtrace,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorDisplay::from_formatter(self, f).fmt(f)
    }
}

impl<'i, S> From<ExpectedLength<'i>> for Expected<'i, S>
where
    S: BacktraceBuilder,
{
    fn from(err: ExpectedLength<'i>) -> Self {
        Self::from_kind(ExpectedKind::Length(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedLength<'i>> for Box<Expected<'i, S>>
where
    S: BacktraceBuilder,
{
    fn from(expected: ExpectedLength<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

impl<'i, S> From<ExpectedValid<'i>> for Expected<'i, S>
where
    S: BacktraceBuilder,
{
    fn from(err: ExpectedValid<'i>) -> Self {
        Self::from_kind(ExpectedKind::Valid(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedValid<'i>> for Box<Expected<'i, S>>
where
    S: BacktraceBuilder,
{
    fn from(expected: ExpectedValid<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

impl<'i, S> From<ExpectedValue<'i>> for Expected<'i, S>
where
    S: BacktraceBuilder,
{
    fn from(err: ExpectedValue<'i>) -> Self {
        Self::from_kind(ExpectedKind::Value(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedValue<'i>> for Box<Expected<'i, S>>
where
    S: BacktraceBuilder,
{
    fn from(expected: ExpectedValue<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_pointer_width = "64", not(feature = "full-backtrace")))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 192);
    }

    #[test]
    #[cfg(all(target_pointer_width = "64", feature = "full-backtrace"))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 216);
    }
}
