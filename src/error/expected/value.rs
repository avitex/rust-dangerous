use crate::fmt;
#[cfg(feature = "retry")]
use crate::input::Input;
use crate::input::{Bytes, MaybeString};

use super::ExpectedContext;

#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

/// An error representing a failed exact value requirement of [`Input`].
#[must_use = "error must be handled"]
pub struct ExpectedValue<'i> {
    pub(crate) input: MaybeString<'i>,
    pub(crate) actual: &'i [u8],
    pub(crate) expected: MaybeString<'i>,
    pub(crate) context: ExpectedContext,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] provided in the context when the error occurred.
    #[inline(always)]
    pub fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }

    /// The [`ExpectedContext`] around the error.
    #[must_use]
    #[inline(always)]
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The [`Input`] that was found.
    #[inline(always)]
    pub fn found(&self) -> Bytes<'i> {
        Bytes::new(self.actual, self.input.bound())
    }

    /// The [`Input`] value that was expected.
    #[inline(always)]
    pub fn expected(&self) -> MaybeString<'i> {
        self.expected.clone()
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

impl<'i> fmt::DisplayBase for ExpectedValue<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("found a different value to the exact expected")
    }
}

impl<'i> fmt::Display for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

#[cfg(feature = "retry")]
impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    #[inline]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let needed = self.expected().into_bytes().len();
            let had = self.found().len();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if the value could never match and `false` if the matching
    /// was incomplete.
    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound()
            || !self
                .expected()
                .into_bytes()
                .as_dangerous()
                .starts_with(self.found().into_bytes().as_dangerous())
    }
}

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for ExpectedValue<'i> {}
