use core::convert::TryInto;
use core::str;

use crate::display::InputDisplay;
use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, ExpectedValue, OperationContext,
    WithContext,
};
use crate::fmt;
use crate::util::{byte, slice, utf8};

use super::{Bound, Input, MaybeString, Private, String};

pub struct Bytes<'i> {
    bytes: &'i [u8],
    #[cfg(feature = "retry")]
    bound: Bound,
}

impl<'i> Bytes<'i> {
    #[cfg(feature = "retry")]
    pub(crate) fn new(bytes: &'i [u8], bound: Bound) -> Self {
        Self { bytes, bound }
    }

    #[cfg(not(feature = "retry"))]
    pub(crate) fn new(bytes: &'i [u8], _bound: Bound) -> Self {
        Self { bytes }
    }

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    pub fn as_dangerous(&self) -> &'i [u8] {
        self.bytes
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    ///
    /// If the underlying byte slice is known to be valid UTF-8 this is will a
    /// cheap operation, otherwise the bytes will be validated.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input is not valid UTF-8.
    pub fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
    {
        let bytes = self.as_dangerous();
        match str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(utf8_err) => {
                let valid_up_to = utf8_err.valid_up_to();
                let span = match utf8_err.error_len() {
                    Some(error_len) => {
                        let error_end = valid_up_to + error_len;
                        &bytes[valid_up_to..error_end]
                    }
                    None => &bytes[valid_up_to..],
                };
                Err(E::from(ExpectedValid {
                    span,
                    input: self.clone().into_maybe_string(),
                    context: ExpectedContext {
                        operation: "convert input to str",
                        expected: "utf-8 code point",
                    },
                    #[cfg(feature = "retry")]
                    retry_requirement: None,
                }))
            }
        }
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
                min: 1,
                max: None,
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
                min: 1,
                max: None,
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
}

impl<'i> Input<'i> for Bytes<'i> {
    #[cfg(feature = "retry")]
    fn bound(&self) -> Bound {
        self.bound
    }

    #[cfg(not(feature = "retry"))]
    fn bound(&self) -> Bound {
        Bound::Both
    }

    #[cfg(feature = "retry")]
    fn into_bound(mut self) -> Self {
        self.bound = Bound::Both;
        self
    }

    #[cfg(not(feature = "retry"))]
    fn into_bound(self) -> Self {
        self
    }

    fn into_bytes(self) -> Bytes<'i> {
        self
    }

    fn into_maybe_string(self) -> MaybeString<'i> {
        MaybeString::Bytes(self)
    }

    fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'i> Bytes<'i> {
    #[inline(always)]
    pub(crate) fn has_prefix(&self, prefix: &[u8]) -> bool {
        self.as_dangerous().starts_with(prefix)
    }

    /// Returns the first byte in the input.
    #[inline(always)]
    pub(crate) fn first_opt(&self) -> Option<u8> {
        self.as_dangerous().first().copied()
    }

    /// Returns the first byte in the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn first<E>(self, operation: &'static str) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.first_opt().ok_or_else(|| {
            E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self.as_dangerous(),
                input: self.into_maybe_string(),
                context: ExpectedContext {
                    operation,
                    expected: "a byte",
                },
            })
        })
    }

    /// Splits the input into two at `mid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `mid > self.len()`.
    #[inline(always)]
    pub(crate) fn split_at<E>(
        self,
        mid: usize,
        operation: &'static str,
    ) -> Result<(Bytes<'i>, Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.clone().split_at_opt(mid).ok_or_else(|| {
            E::from(ExpectedLength {
                min: mid,
                max: None,
                span: self.as_dangerous(),
                input: self.into_maybe_string(),
                context: ExpectedContext {
                    operation,
                    expected: "enough input",
                },
            })
        })
    }

    #[inline(always)]
    pub(crate) fn split_prefix<E>(
        self,
        prefix: &'i [u8],
        operation: &'static str,
    ) -> Result<(Bytes<'i>, Bytes<'i>), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        let actual = match self.clone().split_at_opt(prefix.len()) {
            Some((head, tail)) if head == prefix => return Ok((head, tail)),
            Some((head, _)) => head.as_dangerous(),
            None => self.as_dangerous(),
        };
        Err(E::from(ExpectedValue {
            actual,
            expected: MaybeString::Bytes(Bytes::new(prefix, Bound::Both)),
            input: self.into_maybe_string(),
            context: ExpectedContext {
                operation,
                expected: "exact value",
            },
        }))
    }

    #[inline(always)]
    pub(crate) fn split_prefix_u8<E>(
        self,
        prefix: u8,
        operation: &'static str,
    ) -> Result<(Bytes<'i>, Bytes<'i>), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        let actual = match self.clone().split_at_opt(1) {
            Some((head, tail)) if head.has_prefix(&[prefix]) => return Ok((head, tail)),
            Some((head, _)) => head.as_dangerous(),
            None => self.as_dangerous(),
        };
        Err(E::from(ExpectedValue {
            actual,
            expected: MaybeString::Bytes(Bytes::new(byte::to_slice(prefix), Bound::Both)),
            input: self.into_maybe_string(),
            context: ExpectedContext {
                operation,
                expected: "exact value",
            },
        }))
    }

    #[inline(always)]
    pub(crate) fn split_prefix_opt(self, prefix: &[u8]) -> (Option<Bytes<'i>>, Bytes<'i>) {
        match self.clone().split_at_opt(prefix.len()) {
            Some((head, tail)) if head == prefix => (Some(head), tail),
            _ => (None, self),
        }
    }

    /// Splits the input into the first byte and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn split_first<E>(self, operation: &'static str) -> Result<(u8, Bytes<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(1, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous()[0], tail)),
            Err(err) => Err(err),
        }
    }

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<F>(self, mut pred: F) -> (Bytes<'i>, Bytes<'i>)
    where
        F: FnMut(u8) -> bool,
    {
        let bytes = self.as_dangerous();
        // For each byte, lets make sure it matches the predicate.
        for (i, byte) in bytes.iter().enumerate() {
            // Check if the byte doesn't match the predicate.
            if !pred(*byte) {
                // Split the input up to, but not including the byte.
                // SAFETY: `i` is always a valid index for bytes, derived from the enumerate iterator.
                let (head, tail) = unsafe { slice::split_at_unchecked(bytes, i) };
                // Because we hit the predicate it doesn't matter if we
                // have more input, this will always return the same.
                // This means we know the input has a bound.
                let head = Bytes::new(head, self.bound().close_end());
                // For the tail we derive the bound constaint from self.
                let tail = Bytes::new(tail, self.bound());
                // Return the split input parts.
                return (head, tail);
            }
        }
        (self.clone(), self.end())
    }

    /// Tries to split the input while the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn try_split_while<F, E>(
        self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(Bytes<'i>, Bytes<'i>), E>
    where
        E: WithContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        // For each byte, lets make sure it matches the predicate.
        for (i, byte) in bytes.iter().enumerate() {
            // Check if the byte doesn't match the predicate.
            if !with_context(self.clone(), OperationContext(operation), || f(*byte))? {
                // Split the input up to, but not including the byte.
                // SAFETY: `i` is always a valid index for bytes, derived from the enumerate iterator.
                let (head, tail) = unsafe { slice::split_at_unchecked(bytes, i) };
                // Because we hit the predicate it doesn't matter if we
                // have more input, this will always return the same.
                // This means we know the head input has a bound.
                let head = Bytes::new(head, self.bound().close_end());
                // For the tail we derive the bound constaint from self.
                let tail = Bytes::new(tail, self.bound());
                // Return the split input parts.
                return Ok((head, tail));
            }
        }
        Ok((self.clone(), self.end()))
    }

    #[inline(always)]
    pub(crate) fn split_str_while<F, E>(
        self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(String<'i>, Bytes<'i>), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> bool,
    {
        self.try_split_str_while(|c| Ok(f(c)), operation)
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
        let mut consumed = chars.forward_valid();
        // For each char, lets make sure it matches the predicate.
        while let Some(result) = chars.next() {
            match result {
                Ok(c) => {
                    // Check if the char doesn't match the predicate.
                    if with_context(self.clone(), OperationContext(operation), || f(c))? {
                        consumed = chars.forward_valid();
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
                Err(utf8_err) => match utf8_err.error_len() {
                    None => {
                        let valid_up_to = consumed.as_bytes().len();
                        let invalid = &bytes[valid_up_to..];
                        // SAFETY: For an error to occur there must be a cause (at
                        // least one byte in an invalid codepoint) so it is safe to
                        // get without checking bounds.
                        let first_invalid = unsafe { slice::first_unchecked(invalid) };
                        return Err(E::from(ExpectedLength {
                            min: utf8::char_len(first_invalid),
                            max: None,
                            span: invalid,
                            input: self.into_maybe_string(),
                            context: ExpectedContext {
                                operation,
                                expected: "complete utf-8 code point",
                            },
                        }));
                    }
                    Some(error_len) => {
                        let valid_up_to = consumed.as_bytes().len();
                        let error_end = valid_up_to + error_len;
                        return Err(E::from(ExpectedValid {
                            span: &bytes[valid_up_to..error_end],
                            input: self.into_maybe_string(),
                            context: ExpectedContext {
                                operation,
                                expected: "utf-8 code point",
                            },
                            #[cfg(feature = "retry")]
                            retry_requirement: None,
                        }));
                    }
                },
            }
        }
        Ok((String::new(consumed, self.bound()), self.end()))
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: use https://github.com/rust-lang/rust/pull/79135 once stable in 1.51

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
}

impl<'i> Private<'i> for Bytes<'i> {
    fn end(self) -> Self {
        Self::new(slice::end(self.as_dangerous()), self.bound().for_end())
    }

    #[cfg(feature = "retry")]
    fn into_unbound(mut self) -> Self {
        self.bound = Bound::None;
        self
    }

    #[cfg(not(feature = "retry"))]
    fn into_unbound(self) -> Self {
        self
    }

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        self.split_bytes_at_opt(mid)
    }

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)> {
        slice::split_at_opt(self.as_dangerous(), mid).map(|(head, tail)| {
            // We split at a known length making the head input bound.
            let head = Bytes::new(head, self.bound().close_end());
            // For the tail we derive the bound constraint from self.
            let tail = Bytes::new(tail, self.bound());
            // Return the split input parts.
            (head, tail)
        })
    }

    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self) {
        let (head, tail) = slice::split_at_unchecked(self.as_dangerous(), mid);
        (
            Bytes::new(head, self.bound().close_end()),
            Bytes::new(tail, self.bound()),
        )
    }
}

impl<'i> Clone for Bytes<'i> {
    fn clone(&self) -> Self {
        Self::new(self.bytes, self.bound())
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
        f.debug_tuple("Bytes").field(&display).finish()
    }
}

impl<'i> fmt::DisplayBase for Bytes<'i> {
    fn fmt<W: fmt::Write + ?Sized>(&self, w: &mut W) -> fmt::Result {
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
