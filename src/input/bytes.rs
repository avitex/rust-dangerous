use core::convert::Infallible;
use core::str;

use crate::display::InputDisplay;
use crate::error::{ExpectedContext, ExpectedLength, ExpectedValid, OperationContext, WithContext};
use crate::fmt;
use crate::reader::BytesReader;
use crate::util::slice;

use super::{Bound, Input, MaybeString, Private};

pub struct Bytes<'i> {
    bytes: &'i [u8],
    bound: Bound,
}

impl<'i> Bytes<'i> {
    pub(crate) fn new(bytes: &'i [u8], bound: Bound) -> Self {
        Self { bytes, bound }
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    pub fn read_all<F, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut BytesReader<'i, E>) -> Result<T, E>,
        E: WithContext<'i>,
        E: From<ExpectedLength<'i>>,
    {
        let mut r = BytesReader::new(self.clone());
        match r.context(OperationContext("read all bytes"), f) {
            Ok(ok) if r.at_end() => Ok(ok),
            Ok(_) => Err(E::from(ExpectedLength {
                min: 0,
                max: Some(0),
                span: r.take_remaining().as_dangerous(),
                input: self.into_maybe_string(),
                context: ExpectedContext {
                    operation: "read all",
                    expected: "no trailing input",
                },
            })),
            Err(err) => Err(err),
        }
    }

    /// Create a reader to read a part of the input and return the rest.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided function does.
    pub fn read_partial<F, T, E>(self, f: F) -> Result<(T, Bytes<'i>), E>
    where
        F: FnOnce(&mut BytesReader<'i, E>) -> Result<T, E>,
        E: WithContext<'i>,
    {
        let mut r = BytesReader::new(self);
        match r.context(OperationContext("read partial bytes"), f) {
            Ok(ok) => Ok((ok, r.take_remaining())),
            Err(err) => Err(err),
        }
    }

    /// Create a reader to read a part of the input and return the rest
    /// without any errors.
    pub fn read_infallible<F, T>(self, f: F) -> (T, Bytes<'i>)
    where
        F: FnOnce(&mut BytesReader<'i, Infallible>) -> T,
    {
        let mut r = BytesReader::new(self);
        let ok = f(&mut r);
        (ok, r.take_remaining())
    }
}

impl<'i> Input<'i> for Bytes<'i> {
    fn bound(&self) -> Bound {
        self.bound
    }

    fn as_dangerous(&self) -> &'i [u8] {
        self.bytes
    }

    fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
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
                    retry_requirement: None,
                }))
            }
        }
    }

    fn into_bytes(self) -> Bytes<'i> {
        self
    }

    fn into_bound(mut self) -> Self {
        self.bound = Bound::Both;
        self
    }

    fn into_maybe_string(self) -> MaybeString<'i> {
        MaybeString::Bytes(self)
    }

    fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self)
    }
}

impl<'i> Private<'i> for Bytes<'i> {
    fn end(self) -> Self {
        Self::new(slice::end(self.as_dangerous()), self.bound.for_end())
    }

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)> {
        self.split_bytes_at_opt(mid)
    }

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)> {
        slice::split_at_opt(self.as_dangerous(), mid).map(|(head, tail)| {
            // We split at a known length making the head input bound.
            let head = Bytes::new(head, self.bound.close_end());
            // For the tail we derive the bound constraint from self.
            let tail = Bytes::new(tail, self.bound);
            // Return the split input parts.
            (head, tail)
        })
    }
}

impl<'i> Clone for Bytes<'i> {
    fn clone(&self) -> Self {
        Self::new(self.bytes, self.bound)
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
        f.debug_tuple("Input").field(&display).finish()
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
