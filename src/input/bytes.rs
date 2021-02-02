use core::convert::TryInto;
use core::slice::Iter as SliceIter;
use core::{iter, str};

use crate::display::InputDisplay;
use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, Length, OperationContext,
    WithContext,
};
use crate::fmt;
use crate::util::{slice, utf8};

use super::{Bound, Input, MaybeString, Private, PrivateExt, String};

/// Raw [`Input`].
#[derive(Clone)]
#[must_use = "input must be consumed"]
pub struct Bytes<'i> {
    value: &'i [u8],
    #[cfg(feature = "retry")]
    bound: Bound,
}

impl<'i> Bytes<'i> {
    #[cfg(feature = "retry")]
    #[inline(always)]
    pub(crate) fn new(value: &'i [u8], bound: Bound) -> Self {
        Self { value, bound }
    }

    #[cfg(not(feature = "retry"))]
    #[inline(always)]
    pub(crate) fn new(value: &'i [u8], _bound: Bound) -> Self {
        Self { value }
    }

    /// Returns the underlying byte slice length.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.as_dangerous().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    /// Returns the occurrences of `needle` within the underlying byte slice.
    ///
    /// It is recommended to enable the `bytecount` dependency when using this
    /// function for better performance.
    #[must_use]
    pub fn count(&self, needle: u8) -> usize {
        #[cfg(feature = "bytecount")]
        {
            bytecount::count(self.as_dangerous(), needle)
        }
        #[cfg(not(feature = "bytecount"))]
        {
            self.as_dangerous()
                .iter()
                .copied()
                .filter(|b| *b == needle)
                .count()
        }
    }

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    #[must_use]
    #[inline(always)]
    pub fn as_dangerous(&self) -> &'i [u8] {
        self.value
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    #[inline(always)]
    pub fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        str::from_utf8(self.as_dangerous()).map_err(|err| {
            self.clone()
                .map_utf8_error(err.error_len(), err.valid_up_to(), "convert input to str")
        })
    }

    /// Returns the underlying byte slice if it is not empty.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty.
    pub fn to_dangerous_non_empty<E>(&self) -> Result<&'i [u8], E>
    where
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                len: Length::AtLeast(1),
                span: self.as_dangerous(),
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

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty or [`ExpectedValid`] if
    /// the input is not valid UTF-8.
    pub fn to_dangerous_non_empty_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                len: Length::AtLeast(1),
                span: self.as_dangerous(),
                input: self.clone().into_maybe_string(),
                context: ExpectedContext {
                    operation: "convert input to non-empty str",
                    expected: "non empty input",
                },
            }))
        } else {
            self.to_dangerous_str()
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    #[inline(always)]
    pub fn into_string<E>(self) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        self.to_dangerous_str()
            .map(|s| String::new(s, self.bound()))
    }
}

impl<'i> Input<'i> for Bytes<'i> {
    #[cfg(feature = "retry")]
    #[inline(always)]
    fn bound(&self) -> Bound {
        self.bound
    }

    #[cfg(not(feature = "retry"))]
    #[inline(always)]
    fn bound(&self) -> Bound {
        Bound::Both
    }

    #[cfg(feature = "retry")]
    #[inline(always)]
    fn into_bound(mut self) -> Self {
        self.bound = Bound::force_close();
        self
    }

    #[cfg(not(feature = "retry"))]
    #[inline(always)]
    fn into_bound(self) -> Self {
        self
    }

    #[inline(always)]
    fn into_bytes(self) -> Bytes<'i> {
        self
    }

    #[inline(always)]
    fn into_maybe_string(self) -> MaybeString<'i> {
        MaybeString::Bytes(self)
    }

    #[inline(always)]
    fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'i> Bytes<'i> {
    #[inline(always)]
    pub(crate) fn split_str_while<F, E>(
        self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(String<'i>, Bytes<'i>), E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> bool,
    {
        let bytes = self.as_dangerous();
        let mut chars = utf8::CharIter::new(bytes);
        let mut consumed = chars.as_forward();
        // For each char, lets make sure it matches the predicate.
        while let Some(result) = chars.next() {
            match result {
                Ok(c) if f(c) => {
                    consumed = chars.as_forward();
                }
                Ok(_) => {
                    // Because we hit the predicate it doesn't matter if we
                    // have more input, this will always return the same.
                    // This means we know the head input has a bound.
                    let head = String::new(consumed, self.bound().close_end());
                    // For the tail we derive the bound constaint from self.
                    let tail = Bytes::new(&bytes[consumed.as_bytes().len()..], self.bound());
                    // Return the split input parts.
                    return Ok((head, tail));
                }
                Err(utf8_err) => {
                    return Err(self.map_utf8_error(
                        utf8_err.error_len(),
                        consumed.as_bytes().len(),
                        operation,
                    ))
                }
            }
        }
        Ok((String::new(consumed, self.bound()), self.end()))
    }

    #[inline(always)]
    pub(crate) fn try_split_str_while<F, E>(
        self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(String<'i>, Bytes<'i>), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        let mut chars = utf8::CharIter::new(bytes);
        let mut consumed = chars.as_forward();
        // For each char, lets make sure it matches the predicate.
        while let Some(result) = chars.next() {
            match result {
                Ok(c) => {
                    // Check if the char doesn't match the predicate.
                    if with_context(self.clone(), OperationContext(operation), || f(c))? {
                        consumed = chars.as_forward();
                    } else {
                        // Because we hit the predicate it doesn't matter if we
                        // have more input, this will always return the same.
                        // This means we know the head input has a bound.
                        let head = String::new(consumed, self.bound().close_end());
                        // For the tail we derive the bound constaint from self.
                        let tail = Bytes::new(&bytes[consumed.as_bytes().len()..], self.bound());
                        // Return the split input parts.
                        return Ok((head, tail));
                    }
                }
                Err(utf8_err) => {
                    return Err(self.map_utf8_error(
                        utf8_err.error_len(),
                        consumed.as_bytes().len(),
                        operation,
                    ))
                }
            }
        }
        Ok((String::new(consumed, self.bound()), self.end()))
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: use `split_array` once stable in 1.51

    #[inline(always)]
    pub(crate) fn split_arr_2<E>(self, operation: &'static str) -> Result<([u8; 2], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(2, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_4<E>(self, operation: &'static str) -> Result<([u8; 4], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(4, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_8<E>(self, operation: &'static str) -> Result<([u8; 8], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(8, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_16<E>(self, operation: &'static str) -> Result<([u8; 16], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(16, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    fn map_utf8_error<E>(
        self,
        error_len: Option<usize>,
        valid_up_to: usize,
        operation: &'static str,
    ) -> E
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        let bytes = self.as_dangerous();
        match error_len {
            None => {
                let invalid = &bytes[valid_up_to..];
                // SAFETY: For an error to occur there must be a cause (at
                // least one byte in an invalid codepoint) so it is safe to
                // get without checking bounds.
                let first_invalid = unsafe { slice::first_unchecked(invalid) };
                E::from(ExpectedLength {
                    len: Length::AtLeast(utf8::char_len(first_invalid)),
                    span: invalid,
                    input: self.into_maybe_string(),
                    context: ExpectedContext {
                        operation,
                        expected: "complete utf-8 code point",
                    },
                })
            }
            Some(error_len) => {
                let error_end = valid_up_to + error_len;
                E::from(ExpectedValid {
                    span: &bytes[valid_up_to..error_end],
                    input: self.into_maybe_string(),
                    context: ExpectedContext {
                        operation,
                        expected: "utf-8 code point",
                    },
                    #[cfg(feature = "retry")]
                    retry_requirement: None,
                })
            }
        }
    }
}

impl<'i> Private<'i> for Bytes<'i> {
    type Token = u8;
    type TokenIter = iter::Enumerate<iter::Copied<SliceIter<'i, u8>>>;

    #[inline(always)]
    fn end(self) -> Self {
        Self::new(slice::end(self.as_dangerous()), self.bound().for_end())
    }

    #[inline(always)]
    fn tokens(self) -> Self::TokenIter {
        self.as_dangerous().iter().copied().enumerate()
    }

    #[cfg(feature = "retry")]
    #[inline(always)]
    fn into_unbound_end(mut self) -> Self {
        self.bound = self.bound.open_end();
        self
    }

    #[cfg(not(feature = "retry"))]
    #[inline(always)]
    fn into_unbound_end(self) -> Self {
        self
    }

    #[inline(always)]
    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        slice::split_at_opt(self.as_dangerous(), mid).map(|(head, tail)| {
            // We split at a known length making the head input bound.
            let head = Bytes::new(head, self.bound().close_end());
            // For the tail we derive the bound constraint from self.
            let tail = Bytes::new(tail, self.bound());
            // Return the split input parts.
            (head, tail)
        })
    }

    #[inline(always)]
    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self) {
        let (head, tail) = slice::split_at_unchecked(self.as_dangerous(), mid);
        (
            Bytes::new(head, self.bound().close_end()),
            Bytes::new(tail, self.bound()),
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
// Equality

impl<'i> PartialEq for Bytes<'i> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_dangerous() == other.as_dangerous()
    }
}

impl<'i> PartialEq<[u8]> for Bytes<'i> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<[u8]> for &Bytes<'i> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<&[u8]> for Bytes<'i> {
    #[inline(always)]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_dangerous() == *other
    }
}

impl<'i> PartialEq<Bytes<'i>> for [u8] {
    #[inline(always)]
    fn eq(&self, other: &Bytes<'i>) -> bool {
        self == other.as_dangerous()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i> fmt::Debug for Bytes<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = InputDisplay::from_formatter(self, f);
        f.debug_struct("Bytes")
            .field("bound", &self.bound())
            .field("value", &display)
            .finish()
    }
}

impl<'i> fmt::DisplayBase for Bytes<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i> fmt::Display for Bytes<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InputDisplay::from_formatter(self, f).fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Zc

#[cfg(feature = "zc")]
unsafe impl<'i> zc::NoInteriorMut for Bytes<'i> {}
