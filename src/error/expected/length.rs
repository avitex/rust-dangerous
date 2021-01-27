use crate::display::byte_count;
use crate::fmt;
use crate::input::{Bytes, MaybeString};

use super::ExpectedContext;

#[cfg(feature = "retry")]
use super::{RetryRequirement, ToRetryRequirement};

// FIXME: consider making this public
pub(crate) enum Length {
    AtLeast(usize),
    Exactly(usize),
}

/// An error representing a failed requirement for a length of
/// [`Input`](crate::Input).
#[must_use = "error must be handled"]
pub struct ExpectedLength<'i> {
    pub(crate) len: Length,
    pub(crate) span: &'i [u8],
    pub(crate) input: MaybeString<'i>,
    pub(crate) context: ExpectedContext,
}

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

    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use
    /// [`ToRetryRequirement::to_retry_requirement()`].
    #[must_use]
    #[inline(always)]
    pub fn min(&self) -> usize {
        match self.len {
            Length::AtLeast(min) | Length::Exactly(min) => min,
        }
    }

    /// The maximum length that was expected in a context, if applicable.
    ///
    /// If max has a value, this signifies the [`Input`] exceeded it in some
    /// way. An example of this would be [`Input::read_all()`], where there was
    /// [`Input`] left over.
    ///
    /// [`Input`]: crate::Input
    /// [`Input::read_all()`]: crate::Input::read_all()
    #[must_use]
    #[inline(always)]
    pub fn max(&self) -> Option<usize> {
        match self.len {
            Length::AtLeast(_) => None,
            Length::Exactly(max) => Some(max),
        }
    }

    /// Returns `true` if an exact length was expected in a context.
    #[inline]
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.exact().is_some()
    }

    /// The exact length that was expected in a context, if applicable.
    ///
    /// Will return a value if `is_exact()` returns `true`.
    #[inline]
    #[must_use]
    pub fn exact(&self) -> Option<usize> {
        match self.len {
            Length::AtLeast(_) => None,
            Length::Exactly(exact) => Some(exact),
        }
    }
}

impl<'i> fmt::Debug for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedLength")
            .field("min", &self.min())
            .field("max", &self.max())
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
        match self.len {
            Length::AtLeast(min) => {
                w.write_str("at least ")?;
                byte_count(w, min)?;
            }
            Length::Exactly(exact) => {
                w.write_str("exactly ")?;
                byte_count(w, exact)?;
            }
        }
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
            let needed = self.min();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }

    /// Returns `true` if `max()` has a value.
    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || self.max().is_some()
    }
}

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for ExpectedLength<'i> {}
