use crate::display::byte_count;
use crate::error::Length;
use crate::fmt;
use crate::input::{Bytes, MaybeString};

use super::ExpectedContext;
#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

/// An error representing a failed requirement for a length of
/// [`Input`](crate::Input).
#[must_use = "error must be handled"]
pub struct ExpectedLength<'i> {
    pub(crate) len: Length,
    pub(crate) span: &'i [u8],
    pub(crate) input: MaybeString<'i>,
    pub(crate) context: ExpectedContext,
}

#[allow(clippy::len_without_is_empty)]
impl<'i> ExpectedLength<'i> {
    /// The [`Input`](crate::Input) provided in the context when the error occurred.
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

    /// The specific part of the [`Input`](crate::Input) that did not meet the
    /// requirement.
    #[inline(always)]
    pub fn span(&self) -> Bytes<'i> {
        Bytes::new(self.span, self.input.bound())
    }

    /// The length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use
    /// [`ToRetryRequirement::to_retry_requirement()`].
    #[inline(always)]
    pub fn len(&self) -> Length {
        self.len
    }
}

impl<'i> fmt::Debug for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedLength")
            .field("len", &self.len())
            .field("input", &self.input())
            .field("span", &self.span())
            .field("context", &self.context())
            .finish()
    }
}

impl<'i> fmt::DisplayBase for ExpectedLength<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("found ")?;
        byte_count(w, self.span().len())?;
        w.write_str(" when ")?;
        self.len.fmt(w)?;
        w.write_str(" was expected")
    }
}

impl<'i> fmt::Display for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

#[cfg(feature = "retry")]
impl<'i> ToRetryRequirement for ExpectedLength<'i> {
    #[inline]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let had = self.span().len();
            let needed = self.len().min();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if `max()` has a value.
    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || self.len().max().is_some()
    }
}

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for ExpectedLength<'i> {}
