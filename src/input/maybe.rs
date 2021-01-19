use crate::display::InputDisplay;
use crate::fmt;

use super::{Bound, Bytes, Input, String};

#[must_use = "input must be consumed"]
pub enum MaybeString<'i> {
    Bytes(Bytes<'i>),
    String(String<'i>),
}

impl<'i> MaybeString<'i> {
    pub fn is_string(&self) -> bool {
        match self {
            Self::Bytes(_) => false,
            Self::String(_) => true,
        }
    }

    #[must_use]
    pub fn is_bound(&self) -> bool {
        self.bound() == Bound::Both
    }

    pub fn bound(&self) -> Bound {
        match self {
            Self::Bytes(v) => v.bound(),
            Self::String(v) => v.bound(),
        }
    }

    pub fn into_bytes(self) -> Bytes<'i> {
        match self {
            Self::Bytes(v) => v.into_bytes(),
            Self::String(v) => v.into_bytes(),
        }
    }

    pub fn display(&self) -> InputDisplay<'i> {
        match self {
            Self::Bytes(v) => v.display(),
            Self::String(v) => v.display(),
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
    fn fmt<W: fmt::Write + ?Sized>(&self, w: &mut W) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for MaybeString<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display().fmt(f)
    }
}
