use crate::display::InputDisplay;
use crate::error::ExpectedValid;
use crate::fmt;

use super::{Bound, Bytes, Input, Private, String};

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

    pub fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
    {
        match self {
            Self::Bytes(v) => v.to_dangerous_str(),
            Self::String(v) => Ok(v.as_dangerous()),
        }
    }
}

impl<'i> Input<'i> for MaybeString<'i> {
    fn bound(&self) -> Bound {
        match self {
            Self::Bytes(v) => v.bound(),
            Self::String(v) => v.bound(),
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

    fn into_unbound(self) -> Self {
        match self {
            Self::Bytes(v) => Self::Bytes(v.into_unbound()),
            Self::String(v) => Self::String(v.into_unbound()),
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
                let is_valid_string_split = v.as_dangerous().is_char_boundary(mid);
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

    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self) {
        match self {
            Self::Bytes(v) => {
                let (head, tail) = v.split_at_byte_unchecked(mid);
                (Self::Bytes(head), Self::Bytes(tail))
            }
            Self::String(v) => {
                let (head, tail) = v.split_at_byte_unchecked(mid);
                (Self::String(head), Self::String(tail))
            }
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
        InputDisplay::from_formatter(self, f)
            .str_hint(self.is_string())
            .fmt(f)
    }
}
