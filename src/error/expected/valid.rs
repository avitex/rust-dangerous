use crate::fmt;
use crate::input::{Bytes, MaybeString};

use super::ExpectedContext;

#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

/// An error representing a failed requirement for a valid
/// [`Input`](crate::Input).
#[must_use = "error must be handled"]
pub struct ExpectedValid<'i> {
    pub(crate) input: MaybeString<'i>,
    pub(crate) span: &'i [u8],
    pub(crate) context: ExpectedContext,
    #[cfg(feature = "retry")]
    pub(crate) retry_requirement: Option<RetryRequirement>,
}

impl<'i> ExpectedValid<'i> {
    /// The [`Input`](crate::Input) provided in the context when the error
    /// occurred.
    #[inline(always)]
    pub fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }

    /// The [`ExpectedContext`] around the error.
    #[inline(always)]
    #[must_use]
    pub fn context(&self) -> ExpectedContext {
        self.context
    }

    /// The specific part of the [`Input`](crate::Input) that did not meet the
    /// requirement.
    #[inline(always)]
    pub fn span(&self) -> Bytes<'i> {
        Bytes::new(self.span, self.input.bound())
    }

    /// A description of what was expected.
    #[inline(always)]
    #[must_use]
    pub fn expected(&self) -> &'static str {
        self.context.expected
    }
}

impl<'i> fmt::Debug for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("ExpectedValid");

        debug.field("input", &self.input());
        debug.field("span", &self.span());
        debug.field("context", &self.context());

        #[cfg(feature = "retry")]
        debug.field("retry_requirement", &self.retry_requirement);

        debug.finish()
    }
}

impl<'i> fmt::DisplayBase for ExpectedValid<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("expected ")?;
        w.write_str(self.context.expected)
    }
}

impl<'i> fmt::Display for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

#[cfg(feature = "retry")]
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
        self.input.is_bound() || self.retry_requirement.is_none()
    }
}
