mod maybe;
mod pattern;
mod prefix;

use core::str;

use crate::display::InputDisplay;
use crate::error::{
    CoreContext, CoreExpected, CoreOperation, ExpectedLength, ExpectedValid, Length,
};
use crate::fmt;
use crate::util::{fast, slice, utf8};

pub use self::maybe::MaybeString;

use super::{Bound, Bytes, Input, Private};

/// UTF-8 [`Input`].
#[derive(Clone)]
#[must_use = "input must be consumed"]
pub struct String<'i> {
    utf8: Bytes<'i>,
}

impl<'i> String<'i> {
    pub(crate) const fn new(s: &'i str, bound: Bound) -> Self {
        Self {
            utf8: Bytes::new(s.as_bytes(), bound),
        }
    }

    /// Returns the number of UTF-8 characters in the string.
    ///
    /// It is recommended to enable the `bytecount` dependency when using this
    /// function for better performance.
    #[must_use]
    pub fn num_chars(&self) -> usize {
        fast::num_chars(self.as_dangerous())
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    /// Returns the underlying string slice.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    #[must_use]
    #[inline(always)]
    pub fn as_dangerous(&self) -> &'i str {
        // SAFETY: string container guarantees valid UTF-8 bytes.
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
                len: Length::AtLeast(1),
                context: CoreContext {
                    span: self.span(),
                    operation: CoreOperation::IntoString,
                    expected: CoreExpected::NonEmpty,
                },
                input: self.clone().into_maybe_string(),
            }))
        } else {
            Ok(self.as_dangerous())
        }
    }

    /// Decodes [`Bytes`] into a UTF-8 [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    #[inline(always)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_utf8<E>(utf8: Bytes<'i>) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        utf8.to_dangerous_str().map(|s| Self::new(s, utf8.bound()))
    }

    /// Construct a `String` from unchecked [`Bytes`].
    ///
    /// # Safety
    ///
    /// Caller must ensure that the provides [`Bytes`] are valid UTF-8.
    #[inline(always)]
    pub unsafe fn from_utf8_unchecked(utf8: Bytes<'i>) -> Self {
        Self { utf8 }
    }
}

impl<'i> Input<'i> for String<'i> {
    type Token = char;

    #[inline(always)]
    fn bound(&self) -> Bound {
        self.utf8.bound()
    }

    #[inline(always)]
    fn into_bytes(self) -> Bytes<'i> {
        self.utf8
    }

    #[inline(always)]
    fn into_string<E>(self) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        Ok(self)
    }

    #[inline(always)]
    fn into_bound(mut self) -> Self {
        self.utf8 = self.utf8.into_bound();
        self
    }

    #[inline(always)]
    fn into_maybe_string(self) -> MaybeString<'i> {
        MaybeString::String(self)
    }

    fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self).str_hint()
    }

    #[inline(always)]
    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        let string = self.as_dangerous();
        let iter = &mut string.chars();
        if iter.nth(mid.saturating_sub(1)).is_some() {
            let byte_mid = string.as_bytes().len() - iter.as_str().as_bytes().len();
            // SAFETY: we take byte_mid as the difference between the parent
            // string and the remaining string left over from the char iterator.
            // This means both the index can only ever be valid and the bytes in
            // between are valid UTF-8.
            Some(unsafe { self.split_at_byte_unchecked(byte_mid) })
        } else {
            None
        }
    }
}

impl<'i> Private<'i, char> for String<'i> {
    type TokenIter = str::Chars<'i>;
    type TokenIndicesIter = str::CharIndices<'i>;

    #[inline(always)]
    fn end(self) -> Self {
        Self {
            utf8: self.utf8.end(),
        }
    }

    #[inline(always)]
    fn tokens(self) -> Self::TokenIter {
        self.as_dangerous().chars()
    }

    #[inline(always)]
    fn tokens_indices(self) -> Self::TokenIndicesIter {
        self.as_dangerous().char_indices()
    }

    #[inline(always)]
    fn into_unbound_end(mut self) -> Self {
        self.utf8 = self.utf8.into_unbound_end();
        self
    }

    #[inline(always)]
    fn verify_token_boundary(&self, index: usize) -> Result<(), CoreExpected> {
        if index > self.byte_len() {
            Err(CoreExpected::EnoughInputFor("char index"))
        } else if self.as_dangerous().is_char_boundary(index) {
            Ok(())
        } else {
            Err(CoreExpected::Valid("char index"))
        }
    }

    #[inline(always)]
    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self) {
        let (head, tail) = slice::split_str_at_unchecked(self.as_dangerous(), mid);
        (
            String::new(head, self.bound().close_end()),
            String::new(tail, self.bound()),
        )
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
        let display = self.display().with_formatter(f);
        f.debug_struct("String")
            .field("bound", &self.bound())
            .field("value", &display)
            .finish()
    }
}

impl<'i> fmt::DisplayBase for String<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for String<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display().with_formatter(f).fmt(f)
    }
}
