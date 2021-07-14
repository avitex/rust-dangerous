use crate::display::InputDisplay;
use crate::fmt;
use crate::input::{Bound, Bytes, Input, Span, String};

/// [`String`] if known UTF-8, [`Bytes`] if not.
#[derive(Clone)]
#[must_use = "input must be consumed"]
pub enum MaybeString<'i> {
    /// The [`Input`] is not known to be UTF-8.
    Bytes(Bytes<'i>),
    /// The [`Input`] is known to be UTF-8.
    String(String<'i>),
}

impl<'i> MaybeString<'i> {
    /// Returns `true` if he [`Input`] is known to be UTF-8.
    #[must_use]
    pub fn is_string(&self) -> bool {
        match self {
            Self::Bytes(_) => false,
            Self::String(_) => true,
        }
    }

    /// Returns a [`Span`] from the start of `self` to the end.
    pub fn span(&self) -> Span {
        match self {
            Self::Bytes(v) => v.span(),
            Self::String(v) => v.span(),
        }
    }

    /// Consumes `self` into [`Bytes`].
    pub fn into_bytes(self) -> Bytes<'i> {
        match self {
            Self::Bytes(v) => v.into_bytes(),
            Self::String(v) => v.into_bytes(),
        }
    }

    /// Returns an [`InputDisplay`] for formatting.
    pub fn display(&self) -> InputDisplay<'i> {
        match self {
            Self::Bytes(v) => v.display(),
            Self::String(v) => v.display(),
        }
    }

    /// Returns `true` if [`Self::bound()`] is [`Bound::StartEnd`].
    #[must_use]
    pub fn is_bound(&self) -> bool {
        self.bound() == Bound::StartEnd
    }

    /// Returns the inner [`Input`]s [`Bound`].
    pub fn bound(&self) -> Bound {
        match self {
            Self::Bytes(v) => v.bound(),
            Self::String(v) => v.bound(),
        }
    }

    pub(crate) fn as_dangerous_bytes(&self) -> &'i [u8] {
        match self {
            Self::Bytes(v) => v.as_dangerous(),
            Self::String(v) => v.as_dangerous().as_bytes(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i> fmt::Debug for MaybeString<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bytes(v) => v.fmt(f),
            Self::String(v) => v.fmt(f),
        }
    }
}

impl<'i> fmt::DisplayBase for MaybeString<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for MaybeString<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display().fmt(f)
    }
}
