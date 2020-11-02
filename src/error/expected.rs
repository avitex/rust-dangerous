use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::display::{ByteCount, ErrorDisplay};
use crate::input::Input;

use super::{
    Context, ContextStack, ContextStackBuilder, Details, ExpectedContext, FromContext,
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
///   `~128 bytes`. This is because the `Expected` structure is `160 - 184
///   bytes` large on 64 bit systems and successful parses may be hindered by
///   the time to move the `Result<T, Expected>` value. By boxing `Expected` the
///   size becomes only `8 bytes`. When in doubt, write a benchmark.
///
/// See [`crate::error`] for additional documentation around the error system.
pub struct Expected<'i, S = ExpectedContextStack> {
    input: Input<'i>,
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
    fn add_context<C>(&mut self, input: Input<'i>, context: C)
    where
        C: Context,
    {
        if self.input.is_within(&input) {
            self.input = input
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
    fn input(&self) -> Input<'i> {
        self.input.clone()
    }

    fn span(&self) -> Input<'i> {
        match &self.kind {
            ExpectedKind::Value(err) => err.found(),
            ExpectedKind::Valid(err) => err.span(),
            ExpectedKind::Length(err) => err.span(),
        }
    }

    fn expected(&self) -> Option<Input<'_>> {
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

impl<'i, S> FromContext<'i> for Expected<'i, S>
where
    S: ContextStackBuilder,
{
    fn from_context<C>(mut self, input: Input<'i>, context: C) -> Self
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
    fn from_context<C>(mut self, input: Input<'i>, context: C) -> Self
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

/// An error representing a failed exact value requirement of [`Input`].
pub struct ExpectedValue<'i> {
    pub(crate) input: Input<'i>,
    pub(crate) actual: &'i [u8],
    pub(crate) expected: &'i [u8],
    pub(crate) context: ExpectedContext,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    #[inline(always)]
    pub fn input(&self) -> Input<'i> {
        self.input.clone()
    }

    /// The [`ExpectedContext`] around the error.
    #[inline(always)]
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The [`Input`] that was found.
    #[inline(always)]
    pub fn found(&self) -> Input<'i> {
        Input::new(self.actual, self.input.is_bound())
    }

    /// The [`Input`] value that was expected.
    #[inline(always)]
    pub fn expected(&self) -> Input<'i> {
        Input::new(self.expected, self.input.is_bound())
    }
}

impl<'i> fmt::Debug for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedValue")
            .field("input", &self.input())
            .field("actual", &self.found())
            .field("expected", &self.expected())
            .field("context", &self.context())
            .finish()
    }
}

impl<'i> fmt::Display for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("found a different value to the exact expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    #[inline]
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
    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || !self.expected().has_prefix(self.found().as_dangerous())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected length error

/// An error representing a failed requirement for a length of [`Input`].
pub struct ExpectedLength<'i> {
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) span: &'i [u8],
    pub(crate) input: Input<'i>,
    pub(crate) context: ExpectedContext,
}

impl<'i> ExpectedLength<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    #[inline(always)]
    pub fn input(&self) -> Input<'i> {
        self.input.clone()
    }

    /// The [`ExpectedContext`] around the error.
    #[inline(always)]
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The specific part of the [`Input`] that did not meet the requirement.
    #[inline(always)]
    pub fn span(&self) -> Input<'i> {
        Input::new(self.span, self.input.is_bound())
    }

    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use
    /// [`ToRetryRequirement::to_retry_requirement()`].
    #[inline(always)]
    pub fn min(&self) -> usize {
        self.min
    }

    /// The maximum length that was expected in a context, if applicable.
    ///
    /// If max has a value, this signifies the [`Input`] exceeded it in some
    /// way. An example of this would be [`Input::read_all()`], where there was
    /// [`Input`] left over.
    #[inline(always)]
    pub fn max(&self) -> Option<usize> {
        self.max
    }

    /// Returns `true` if an exact length was expected in a context.
    #[inline]
    pub fn is_exact(&self) -> bool {
        Some(self.min) == self.max
    }

    /// The exact length that was expected in a context, if applicable.
    ///
    /// Will return a value if `is_exact()` returns `true`.
    #[inline]
    pub fn exact(&self) -> Option<usize> {
        if self.is_exact() {
            self.max
        } else {
            None
        }
    }
}

impl<'i> fmt::Debug for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedValue")
            .field("min", &self.min())
            .field("max", &self.max())
            .field("input", &self.input())
            .field("span", &self.span())
            .field("context", &self.context())
            .finish()
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
    #[inline]
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
    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || self.max.is_some()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected valid error

/// An error representing a failed requirement for a valid [`Input`].
pub struct ExpectedValid<'i> {
    pub(crate) input: Input<'i>,
    pub(crate) span: &'i [u8],
    pub(crate) context: ExpectedContext,
    pub(crate) retry_requirement: Option<RetryRequirement>,
}

impl<'i> ExpectedValid<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    #[inline(always)]
    pub fn input(&self) -> Input<'i> {
        self.input.clone()
    }

    /// The [`ExpectedContext`] around the error.
    #[inline(always)]
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The specific part of the [`Input`] that did not meet the requirement.
    #[inline(always)]
    pub fn span(&self) -> Input<'i> {
        Input::new(self.span, self.input.is_bound())
    }

    /// A description of what was expected.
    #[inline(always)]
    pub fn expected(&self) -> &'static str {
        self.context.expected
    }
}

impl<'i> fmt::Debug for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedValue")
            .field("input", &self.input())
            .field("span", &self.span())
            .field("context", &self.context())
            .field("retry_requirement", &self.retry_requirement)
            .finish()
    }
}

impl<'i> fmt::Display for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}", self.context.expected)
    }
}

impl<'i> ToRetryRequirement for ExpectedValid<'i> {
    #[inline]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            self.retry_requirement
        }
    }

    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || self.retry_requirement.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_pointer_width = "64", not(feature = "full-context")))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 160);
    }

    #[test]
    #[cfg(all(target_pointer_width = "64", feature = "full-context"))]
    fn test_expected_size() {
        // Update the docs if this value changes.
        assert_eq!(core::mem::size_of::<Expected<'_>>(), 184);
    }
}
