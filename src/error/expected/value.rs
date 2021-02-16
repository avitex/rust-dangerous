use crate::error::Value;
use crate::error::{CoreContext, RetryRequirement, ToRetryRequirement};
use crate::fmt;
use crate::input::MaybeString;

/// An error representing a failed exact value requirement of
/// [`Input`](crate::Input).
#[must_use = "error must be handled"]
pub struct ExpectedValue<'i> {
    pub(crate) expected: Value<'i>,
    pub(crate) context: CoreContext,
    pub(crate) input: MaybeString<'i>,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`](crate::Input) value that was expected.
    #[inline(always)]
    pub fn expected(&self) -> Value<'i> {
        self.expected
    }

    /// The [`CoreContext`] around the error.
    #[must_use]
    #[inline(always)]
    pub fn context(&self) -> CoreContext {
        self.context
    }

    /// The [`Input`](crate::Input) provided in the context when the error occurred.
    #[inline(always)]
    pub fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }
}

impl<'i> fmt::Debug for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedValue")
            .field("expected", &self.expected())
            .field("context", &self.context().debug_for(self.input()))
            .field("input", &self.input())
            .finish()
    }
}

impl<'i> fmt::DisplayBase for ExpectedValue<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        if self.is_fatal() {
            w.write_str("found a different value to the exact expected")
        } else {
            w.write_str("not enough input to match expected value")
        }
    }
}

impl<'i> fmt::Display for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    #[inline]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let needed = self.expected().as_bytes().len();
            let had = self.context.span.len();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if the value could never match and `false` if the matching
    /// was incomplete.
    #[inline]
    fn is_fatal(&self) -> bool {
        if self.input.is_bound() {
            return true;
        }
        match self.context.span.of(self.input.as_dangerous_bytes()) {
            Some(found) => !self.expected().as_bytes().starts_with(found),
            None => true,
        }
    }
}
