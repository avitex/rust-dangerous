use crate::display::InputDisplay;
use crate::error::{ExpectedContext, ExpectedLength};
use crate::fmt;
use crate::util::{slice, utf8};

use super::{Bound, Bytes, Input, MaybeString, Private};

pub struct String<'i> {
    utf8: Bytes<'i>,
}

impl<'i> String<'i> {
    pub(crate) fn new(s: &'i str, bound: Bound) -> Self {
        Self {
            utf8: Bytes::new(s.as_bytes(), bound),
        }
    }

    /// Returns the underlying string slice.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    pub fn as_dangerous(&self) -> &'i str {
        unsafe { utf8::from_unchecked(self.utf8.as_dangerous()) }
    }

    /// Returns the underlying string slice if it is not empty.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty.
    pub fn to_dangerous_non_empty<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self.as_dangerous().as_bytes(),
                input: self.clone().into_maybe_string(),
                context: ExpectedContext {
                    operation: "convert input to non-empty slice",
                    expected: "non-empty input",
                },
            }))
        } else {
            Ok(self.as_dangerous())
        }
    }

    /// Construct a `String` from unchecked [`Bytes`].
    ///
    /// # Safety
    ///
    /// Caller must ensure that the provides [`Bytes`] are valid UTF-8.
    pub unsafe fn from_bytes_unchecked(utf8: Bytes<'i>) -> Self {
        Self { utf8 }
    }
}

impl<'i> Input<'i> for String<'i> {
    fn bound(&self) -> Bound {
        self.utf8.bound()
    }

    fn into_bytes(self) -> Bytes<'i> {
        self.utf8
    }

    fn into_bound(mut self) -> Self {
        self.utf8 = self.utf8.into_bound();
        self
    }

    fn into_maybe_string(self) -> MaybeString<'i> {
        MaybeString::String(self)
    }

    fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self).str_hint(true)
    }
}

impl<'i> Private<'i> for String<'i> {
    fn end(self) -> Self {
        Self {
            utf8: self.utf8.end(),
        }
    }

    fn into_unbound(mut self) -> Self {
        self.utf8 = self.utf8.into_unbound();
        self
    }

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        let string = self.as_dangerous();
        let iter = &mut string.chars();
        if iter.nth(mid.saturating_sub(1)).is_some() {
            let mid = string.as_bytes().len() - iter.as_str().as_bytes().len();
            Some(unsafe { self.split_at_byte_unchecked(mid) })
        } else {
            None
        }
    }

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)> {
        self.utf8.split_bytes_at_opt(mid)
    }

    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self) {
        let (head, tail) = slice::split_str_at_unchecked(self.as_dangerous(), mid);
        (
            String::new(head, self.bound().close_end()),
            String::new(tail, self.bound()),
        )
    }
}

impl<'i> Clone for String<'i> {
    fn clone(&self) -> Self {
        Self {
            utf8: self.utf8.clone(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Equality

impl<'i> PartialEq for String<'i> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_dangerous() == other.as_dangerous()
    }
}

impl<'i> PartialEq<str> for String<'i> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<str> for &String<'i> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<&str> for String<'i> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.as_dangerous() == *other
    }
}

impl<'i> PartialEq<String<'i>> for str {
    #[inline(always)]
    fn eq(&self, other: &String<'i>) -> bool {
        self == other.as_dangerous()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i> fmt::Debug for String<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = InputDisplay::from_formatter(self, f).str_hint(true);
        f.debug_tuple("String").field(&display).finish()
    }
}

impl<'i> fmt::DisplayBase for String<'i> {
    fn fmt<W: fmt::Write + ?Sized>(&self, w: &mut W) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for String<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InputDisplay::from_formatter(self, f).str_hint(true).fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Zc

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for String<'i> {}

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for MaybeString<'i> {}
