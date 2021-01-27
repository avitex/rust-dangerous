use crate::display::InputDisplay;
use crate::fmt;

use super::{Bound, Bytes, Input, String};

/// [`String`] if known UTF-8, [`Bytes`] if not.
#[must_use]
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

    #[cfg(feature = "retry")]
    pub(crate) fn is_bound(&self) -> bool {
        self.bound() == Bound::Both
    }

    pub(crate) fn bound(&self) -> Bound {
        match self {
            Self::Bytes(v) => v.bound(),
            Self::String(v) => v.bound(),
        }
    }
}

impl<'i> Clone for MaybeString<'i> {
    fn clone(&self) -> Self {
        match self {
            Self::Bytes(v) => Self::Bytes(v.clone()),
            Self::String(v) => Self::String(v.clone()),
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
