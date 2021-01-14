use crate::display::InputDisplay;
use crate::error::ExpectedValid;
use crate::fmt;

use super::{Bound, Bytes, Input, Private, String};

pub enum MaybeString<'i> {
    Bytes(Bytes<'i>),
    String(String<'i>),
}

impl<'i> MaybeString<'i> {
    /// Returns an [`InputDisplay`] for formatting.
    #[inline(always)]
    pub fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self)
    }
}

impl<'i> Input<'i> for MaybeString<'i> {
    fn bound(&self) -> Bound {
        match self {
            Self::Bytes(v) => v.bound(),
            Self::String(v) => v.bound(),
        }
    }

    fn as_dangerous(&self) -> &'i [u8] {
        match self {
            Self::Bytes(v) => v.as_dangerous(),
            Self::String(v) => v.as_dangerous(),
        }
    }

    fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
    {
        match self {
            Self::Bytes(v) => v.to_dangerous_str(),
            Self::String(v) => v.to_dangerous_str(),
        }
    }

    fn into_bytes(self) -> Bytes<'i> {
        match self {
            Self::Bytes(v) => v.into_bytes(),
            Self::String(v) => v.into_bytes(),
        }
    }

    fn into_bound(self) -> Self {
        match self {
            Self::Bytes(v) => Self::Bytes(v.into_bound()),
            Self::String(v) => Self::String(v.into_bound()),
        }
    }

    fn into_maybe_string(self) -> MaybeString<'i> {
        self
    }

    fn display(&self) -> InputDisplay<'i> {
        match self {
            Self::Bytes(v) => v.display(),
            Self::String(v) => v.display(),
        }
    }
}

impl<'i> Private<'i> for MaybeString<'i> {
    fn end(self) -> Self {
        match self {
            Self::Bytes(v) => Self::Bytes(v.end()),
            Self::String(v) => Self::String(v.end()),
        }
    }

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        if self.len() < mid {
            return None;
        }
        match self {
            Self::Bytes(v) => v
                .split_bytes_at_opt(mid)
                .map(|(head, tail)| (Self::Bytes(head), Self::Bytes(tail))),
            Self::String(v) => {
                let is_valid_string_split = v.as_dangerous_str().is_char_boundary(mid);
                v.split_bytes_at_opt(mid).map(|(head, tail)| {
                    if is_valid_string_split {
                        let head = unsafe { String::from_bytes_unchecked(head) };
                        let tail = unsafe { String::from_bytes_unchecked(tail) };
                        (Self::String(head), Self::String(tail))
                    } else {
                        (Self::Bytes(head), Self::Bytes(tail))
                    }
                })
            }
        }
    }

    fn split_bytes_at_opt(self, index: usize) -> Option<(Bytes<'i>, Bytes<'i>)> {
        self.into_bytes().split_bytes_at_opt(index)
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
        let display = InputDisplay::from_formatter(self, f).str_hint(true);
        f.debug_tuple("Input").field(&display).finish()
    }
}

impl<'i> fmt::DisplayBase for MaybeString<'i> {
    fn fmt<W: fmt::Write + ?Sized>(&self, w: &mut W) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for MaybeString<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InputDisplay::from_formatter(self, f).str_hint(true).fmt(f)
    }
}
