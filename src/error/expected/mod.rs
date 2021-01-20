mod length;
mod valid;
mod value;

pub use self::length::ExpectedLength;
pub use self::valid::ExpectedValid;
pub use self::value::ExpectedValue;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::display::ErrorDisplay;
use crate::fmt;
use crate::input::{Bytes, Input, MaybeString};

use super::{Context, ContextStack, ContextStackBuilder, Details, ExpectedContext, WithContext};

#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

#[cfg(feature = "full-context")]
type ExpectedContextStack = crate::error::FullContextStack;
#[cfg(not(feature = "full-context"))]
type ExpectedContextStack = crate::error::RootContextStack;

/// A catch-all error for all expected errors supported in this crate.
///
/// - Enable the `full-context` feature (enabled by default), for full-context
///   stacks.
/// - It is generally recommended for better performance to box `Expected` if
///   the structures being returned from parsing are smaller than or equal to
///   `~128 bytes`. This is because the `Expected` structure is `184 - 208
///   bytes` large on 64 bit systems and successful parses may be hindered by
///   the time to move the `Result<T, Expected>` value. By boxing `Expected` the
///   size becomes only `8 bytes`. When in doubt, write a benchmark.
///
/// See [`crate::error`] for additional documentation around the error system.
#[must_use = "error must be handled"]
pub struct Expected<'i, S = ExpectedContextStack> {
    input: MaybeString<'i>,
    stack: S,
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
    S: ContextStack,
{
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<'_, Self> {
        ErrorDisplay::new(self)
    }
}

impl<'i, S> Expected<'i, S>
where
    S: ContextStackBuilder,
{
    #[inline(always)]
    fn add_context(&mut self, input: impl Input<'i>, context: impl Context) {
        if self.input.clone().into_bytes().is_within(&input) {
            self.input = input.into_maybe_string()
        }
        self.stack.push(context);
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
            stack: S::from_root(context),
        }
    }
}

impl<'i, S> Details<'i> for Expected<'i, S>
where
    S: ContextStack,
{
    fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }

    fn span(&self) -> Bytes<'i> {
        match &self.kind {
            ExpectedKind::Value(err) => err.found(),
            ExpectedKind::Valid(err) => err.span(),
            ExpectedKind::Length(err) => err.span(),
        }
    }

    fn expected(&self) -> Option<MaybeString<'i>> {
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

    fn context_stack(&self) -> &dyn ContextStack {
        &self.stack
    }
}

#[cfg(feature = "retry")]
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

#[cfg(all(feature = "alloc", feature = "retry"))]
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
    S: ContextStackBuilder,
{
    fn with_context(mut self, input: impl Input<'i>, context: impl Context) -> Self {
        self.add_context(input, context);
        self
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> WithContext<'i> for Box<Expected<'i, S>>
where
    S: ContextStackBuilder,
{
    fn with_context(mut self, input: impl Input<'i>, context: impl Context) -> Self {
        self.add_context(input, context);
        self
    }
}

impl<'i, S> fmt::Debug for Expected<'i, S>
where
    S: ContextStack,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorDisplay::from_formatter(self, f).banner(true).fmt(f)
    }
}

impl<'i, S> fmt::Display for Expected<'i, S>
where
    S: ContextStack,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorDisplay::from_formatter(self, f).fmt(f)
    }
}

impl<'i, S> From<ExpectedLength<'i>> for Expected<'i, S>
where
    S: ContextStackBuilder,
{
    fn from(err: ExpectedLength<'i>) -> Self {
        Self::from_kind(ExpectedKind::Length(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedLength<'i>> for Box<Expected<'i, S>>
where
    S: ContextStackBuilder,
{
    fn from(expected: ExpectedLength<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

impl<'i, S> From<ExpectedValid<'i>> for Expected<'i, S>
where
    S: ContextStackBuilder,
{
    fn from(err: ExpectedValid<'i>) -> Self {
        Self::from_kind(ExpectedKind::Valid(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedValid<'i>> for Box<Expected<'i, S>>
where
    S: ContextStackBuilder,
{
    fn from(expected: ExpectedValid<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

impl<'i, S> From<ExpectedValue<'i>> for Expected<'i, S>
where
    S: ContextStackBuilder,
{
    fn from(err: ExpectedValue<'i>) -> Self {
        Self::from_kind(ExpectedKind::Value(err))
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> From<ExpectedValue<'i>> for Box<Expected<'i, S>>
where
    S: ContextStackBuilder,
{
    fn from(expected: ExpectedValue<'i>) -> Box<Expected<'i, S>> {
        Box::new(expected.into())
    }
}

#[cfg(feature = "std")]
impl<'i, S> std::error::Error for Expected<'i, S> where S: ContextStack {}

#[cfg(feature = "zc")]
unsafe impl<'i, S> zc::NoInteriorMut for Expected<'i, S> where S: zc::NoInteriorMut {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_pointer_width = "64", not(feature = "full-context")))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 184);
    }

    #[test]
    #[cfg(all(target_pointer_width = "64", feature = "full-context"))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 208);
    }
}
