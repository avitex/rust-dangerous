use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::display::{ByteCount, ErrorDisplay};
use crate::input::{input, Input};

use super::{
    Context, ContextStack, ContextStackBuilder, Details, ExpectedContext, FromContext, IntoFatal,
    RetryRequirement, ToRetryRequirement,
};

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
///   `~128 bytes`. This is because the `Expected` structure is `136 - 160
///   bytes` large on 64 bit systems and successful parses may be hindered by
///   the time to move the `Result<T, Expected>` value. By boxing `Expected` the
///   size becomes only `8 bytes`. When in doubt, write a benchmark.
///
/// See [`crate::error`] for additional documentation around the error system.
pub struct Expected<'i, S = ExpectedContextStack> {
    stack: S,
    fatal: bool,
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
    fn add_context<C>(&mut self, input: &'i Input, context: C)
    where
        C: Context,
    {
        let current_input = match &mut self.kind {
            ExpectedKind::Value(err) => &mut err.input,
            ExpectedKind::Valid(err) => &mut err.input,
            ExpectedKind::Length(err) => &mut err.input,
        };
        if current_input.is_within(input) {
            *current_input = input
        }
        self.stack.push(context);
    }

    #[inline]
    fn from_kind(kind: ExpectedKind<'i>) -> Self {
        let context = match &kind {
            ExpectedKind::Valid(err) => err.context(),
            ExpectedKind::Value(err) => err.context(),
            ExpectedKind::Length(err) => err.context(),
        };

        Self {
            kind,
            fatal: false,
            stack: S::from_root(context),
        }
    }
}

impl<'i, S> Details<'i> for Expected<'i, S>
where
    S: ContextStack,
{
    fn input(&self) -> &'i Input {
        match &self.kind {
            ExpectedKind::Value(err) => err.input(),
            ExpectedKind::Valid(err) => err.input(),
            ExpectedKind::Length(err) => err.input(),
        }
    }

    fn span(&self) -> &'i Input {
        match &self.kind {
            ExpectedKind::Value(err) => err.found(),
            ExpectedKind::Valid(err) => err.span(),
            ExpectedKind::Length(err) => err.span(),
        }
    }

    fn expected(&self) -> Option<&Input> {
        match &self.kind {
            ExpectedKind::Value(err) => Some(err.expected()),
            ExpectedKind::Valid(_) | ExpectedKind::Length(_) => None,
        }
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExpectedKind::Value(err) => fmt::Display::fmt(err, f),
            ExpectedKind::Valid(err) => fmt::Display::fmt(err, f),
            ExpectedKind::Length(err) => fmt::Display::fmt(err, f),
        }
    }

    fn context_stack(&self) -> &dyn ContextStack {
        &self.stack
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
        if self.fatal {
            return true;
        }
        match &self.kind {
            ExpectedKind::Value(err) => err.is_fatal(),
            ExpectedKind::Valid(err) => err.is_fatal(),
            ExpectedKind::Length(err) => err.is_fatal(),
        }
    }
}

impl<'i, S> IntoFatal for Expected<'i, S> {
    fn into_fatal(mut self) -> Self {
        self.fatal = true;
        self
    }
}

impl<'i, S> FromContext<'i> for Expected<'i, S>
where
    S: ContextStackBuilder,
{
    fn from_context<C>(mut self, input: &'i Input, context: C) -> Self
    where
        C: Context,
    {
        self.add_context(input, context);
        self
    }
}

#[cfg(feature = "alloc")]
impl<'i, S> FromContext<'i> for Box<Expected<'i, S>>
where
    S: ContextStackBuilder,
{
    fn from_context<C>(mut self, input: &'i Input, context: C) -> Self
    where
        C: Context,
    {
        self.add_context(input, context);
        self
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

impl<'i, S> fmt::Debug for Expected<'i, S>
where
    S: ContextStack,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ErrorDisplay::from_formatter(self, f).banner(true).fmt(f)
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

///////////////////////////////////////////////////////////////////////////////
// Expected value error

#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
pub(crate) enum Value<'a> {
    Byte(u8),
    Bytes(&'a [u8]),
}

impl<'i> Value<'i> {
    pub(crate) fn as_input(&self) -> &Input {
        match self {
            Self::Byte(b) => Input::from_u8(b),
            Self::Bytes(bytes) => input(bytes),
        }
    }
}

/// An error representing a failed exact value requirement of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValue<'i> {
    pub(crate) input: &'i Input,
    pub(crate) actual: &'i Input,
    pub(crate) expected: Value<'i>,
    pub(crate) context: ExpectedContext,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    pub fn input(&self) -> &'i Input {
        self.input
    }

    /// The [`ExpectedContext`] around the error.
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The [`Input`] that was found.
    pub fn found(&self) -> &'i Input {
        self.actual
    }

    /// The [`Input`] value that was expected.
    pub fn expected(&self) -> &Input {
        self.expected.as_input()
    }
}

impl<'i> fmt::Display for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("found a different value to the exact expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let needed = self.expected().len();
            let had = self.found().len();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if the value could never match and `false` if the matching
    /// was incomplete.
    fn is_fatal(&self) -> bool {
        !self.expected().has_prefix(self.found().as_dangerous())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected length error

/// An error representing a failed requirement for a length of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedLength<'i> {
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) context: ExpectedContext,
}

impl<'i> ExpectedLength<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    pub fn input(&self) -> &'i Input {
        self.input
    }

    /// The [`ExpectedContext`] around the error.
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The specific part of the [`Input`] that did not meet the requirement.
    pub fn span(&self) -> &'i Input {
        self.span
    }

    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use
    /// [`ToRetryRequirement::to_retry_requirement()`].
    pub fn min(&self) -> usize {
        self.min
    }

    /// The maximum length that was expected in a context, if applicable.
    ///
    /// If max has a value, this signifies the [`Input`] exceeded it in some
    /// way. An example of this would be [`Input::read_all()`], where there was
    /// [`Input`] left over.
    pub fn max(&self) -> Option<usize> {
        self.max
    }

    /// Returns `true` if an exact length was expected in a context.
    pub fn is_exact(&self) -> bool {
        Some(self.min) == self.max
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
}

impl<'i> fmt::Display for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "found {} when ", ByteCount(self.span().len()))?;
        match (self.min(), self.max()) {
            (0, Some(max)) => write!(f, "at most {}", ByteCount(max)),
            (min, None) => write!(f, "at least {}", ByteCount(min)),
            (min, Some(max)) if min == max => write!(f, "exactly {}", ByteCount(min)),
            (min, Some(max)) => write!(
                f,
                "at least {} and at most {}",
                ByteCount(min),
                ByteCount(max)
            ),
        }?;
        write!(f, " was expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedLength<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let had = self.span().len();
            let needed = self.min();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if `max()` has a value.
    fn is_fatal(&self) -> bool {
        self.max.is_some()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected valid error

/// An error representing a failed requirement for a valid [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValid<'i> {
    pub(crate) input: &'i Input,
    pub(crate) span: &'i Input,
    pub(crate) context: ExpectedContext,
    pub(crate) retry_requirement: Option<RetryRequirement>,
}

impl<'i> ExpectedValid<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    pub fn input(&self) -> &'i Input {
        self.input
    }

    /// The [`ExpectedContext`] around the error.
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The specific part of the [`Input`] that did not meet the requirement.
    pub fn span(&self) -> &'i Input {
        self.span
    }

    /// A description of what was expected.
    pub fn expected(&self) -> &'static str {
        self.context.expected
    }
}

impl<'i> fmt::Display for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}", self.context.expected)
    }
}

impl<'i> ToRetryRequirement for ExpectedValid<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_pointer_width = "64", not(feature = "full-context")))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 136);
    }

    #[test]
    #[cfg(all(target_pointer_width = "64", feature = "full-context"))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 160);
    }
}
