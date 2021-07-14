use core::slice::Iter as SliceIter;
use core::{iter, str};

use crate::display::InputDisplay;
use crate::error::{
    with_context, CoreContext, CoreExpected, CoreOperation, ExpectedLength, ExpectedValid, Length,
    WithContext,
};
use crate::fmt;
use crate::util::{fast, slice, utf8};

use super::{Bound, Input, MaybeString, Private, PrivateExt, String};

/// Raw [`Input`].
#[derive(Clone)]
#[must_use = "input must be consumed"]
pub struct Bytes<'i> {
    value: &'i [u8],
    bound: Bound,
}

impl<'i> Bytes<'i> {
    #[inline(always)]
    pub(crate) fn new(value: &'i [u8], bound: Bound) -> Self {
        Self { value, bound }
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
        fast::count_u8(needle, self.as_dangerous())
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
            self.clone().map_utf8_error(
                err.error_len(),
                err.valid_up_to(),
                CoreOperation::IntoString,
            )
        })
    }
}

impl<'i> Input<'i> for Bytes<'i> {
    #[inline(always)]
    fn bound(&self) -> Bound {
        self.bound
    }

    #[inline(always)]
    fn into_bound(mut self) -> Self {
        self.bound = Bound::force_close();
        self
    }

    #[inline(always)]
    fn into_bytes(self) -> Bytes<'i> {
        self
    }

    #[inline(always)]
    fn into_string<E>(self) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        String::from_utf8(self)
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
        operation: CoreOperation,
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
        operation: CoreOperation,
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
                    let should_continue = with_context(
                        CoreContext::from_operation(operation, self.span()),
                        self.clone(),
                        || f(c),
                    )?;
                    if should_continue {
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

    #[inline(always)]
    pub(crate) fn split_array<E, const N: usize>(
        self,
        operation: CoreOperation,
    ) -> Result<([u8; N], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.split_array_ref(operation)
            .map(|(arr, tail)| (*arr, tail))
    }

    #[inline(always)]
    pub(crate) fn split_array_ref<E, const N: usize>(
        self,
        operation: CoreOperation,
    ) -> Result<(&'i [u8; N], Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(N, operation) {
            Ok((head, tail)) => {
                // SAFETY: safe as we took only N amount.
                let arr = unsafe { slice::slice_to_array_unchecked(head.as_dangerous()) };
                Ok((arr, tail))
            }
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_array_opt<const N: usize>(self) -> Option<([u8; N], Bytes<'i>)> {
        self.split_array_ref_opt().map(|(arr, tail)| (*arr, tail))
    }

    #[inline(always)]
    pub(crate) fn split_array_ref_opt<const N: usize>(self) -> Option<(&'i [u8; N], Bytes<'i>)> {
        self.split_at_opt(N).map(|(head, tail)| {
            // SAFETY: safe as we took only N amount.
            let arr = unsafe { slice::slice_to_array_unchecked(head.as_dangerous()) };
            (arr, tail)
        })
    }

    fn map_utf8_error<E>(
        self,
        error_len: Option<usize>,
        valid_up_to: usize,
        operation: CoreOperation,
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
                    context: CoreContext {
                        span: invalid.into(),
                        operation,
                        expected: CoreExpected::EnoughInputFor("utf-8 code point"),
                    },
                    input: self.into_maybe_string(),
                })
            }
            Some(error_len) => {
                let error_end = valid_up_to + error_len;
                E::from(ExpectedValid {
                    retry_requirement: None,
                    context: CoreContext {
                        span: bytes[valid_up_to..error_end].into(),
                        operation,
                        expected: CoreExpected::Valid("utf-8 code point"),
                    },
                    input: self.into_maybe_string(),
                })
            }
        }
    }
}

impl<'i> Private<'i> for Bytes<'i> {
    type Token = u8;
    type TokenIter = iter::Copied<SliceIter<'i, u8>>;
    type TokenIndicesIter = iter::Enumerate<iter::Copied<SliceIter<'i, u8>>>;

    #[inline(always)]
    fn end(self) -> Self {
        Self::new(slice::end(self.as_dangerous()), self.bound().for_end())
    }

    #[inline(always)]
    fn tokens(self) -> Self::TokenIter {
        self.as_dangerous().iter().copied()
    }

    #[inline(always)]
    fn tokens_indices(self) -> Self::TokenIndicesIter {
        self.as_dangerous().iter().copied().enumerate()
    }

    #[inline(always)]
    fn into_unbound_end(mut self) -> Self {
        self.bound = self.bound.open_end();
        self
    }

    #[inline(always)]
    fn verify_token_boundary(&self, index: usize) -> Result<(), CoreExpected> {
        if index > self.len() {
            Err(CoreExpected::EnoughInputFor("byte index"))
        } else {
            Ok(())
        }
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
        let display = self.display().with_formatter(f);
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
        self.display().with_formatter(f).fmt(f)
    }
}
