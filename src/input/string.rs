use crate::display::InputDisplay;
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

    /// Construct a `String` from unchecked [`Bytes`].
    ///
    /// # Safety
    ///
    /// Caller must ensure that the provides [`Bytes`] are valid UTF-8.
    pub unsafe fn from_bytes_unchecked(utf8: Bytes<'i>) -> Self {
        Self { utf8 }
    }

    /// Returns the underlying string slice.
    ///
    /// See [`Input::as_dangerous`] for naming.
    pub fn as_dangerous_str(&self) -> &'i str {
        unsafe { utf8::from_unchecked(self.as_dangerous()) }
    }
}

impl<'i> Input<'i> for String<'i> {
    fn bound(&self) -> Bound {
        self.utf8.bound()
    }

    fn as_dangerous(&self) -> &'i [u8] {
        self.utf8.as_dangerous()
    }

    fn to_dangerous_str<E>(&self) -> Result<&'i str, E> {
        Ok(self.as_dangerous_str())
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

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        let string = self.as_dangerous_str();
        let iter = &mut string.chars();
        if iter.nth(mid.saturating_sub(1)).is_some() {
            let mid = string.as_bytes().len() - iter.as_str().as_bytes().len();
            let (head, tail) = unsafe { slice::split_str_at_unchecked(string, mid) };
            let head = String::new(head, self.bound().close_end());
            let tail = String::new(tail, self.bound());
            Some((head, tail))
        } else {
            None
        }
    }

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)> {
        self.utf8.split_bytes_at_opt(mid)
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
        self.as_dangerous_str() == other.as_dangerous_str()
    }
}

impl<'i> PartialEq<str> for String<'i> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_dangerous_str() == other
    }
}

impl<'i> PartialEq<str> for &String<'i> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_dangerous_str() == other
    }
}

impl<'i> PartialEq<&str> for String<'i> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.as_dangerous_str() == *other
    }
}

impl<'i> PartialEq<String<'i>> for str {
    #[inline(always)]
    fn eq(&self, other: &String<'i>) -> bool {
        self == other.as_dangerous_str()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i> fmt::Debug for String<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = InputDisplay::from_formatter(self, f).str_hint(true);
        f.debug_tuple("Input").field(&display).finish()
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
