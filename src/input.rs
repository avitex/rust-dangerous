use core::ops::Range;
use core::{fmt, str};

use crate::error::{Error, ExpectedLength, ExpectedValid, SealedContext};
use crate::input_display::InputDisplay;
use crate::reader::Reader;

/// Constructs a new `Input` from a byte slice.
///
/// It is recommended to use this directly from the crate as `dangerous::input()`,
/// not as an import via `use` as shown below, as you lose the discoverability.
///
/// ```
/// use dangerous::input; // bad
///
/// dangerous::input(b"hello"); // do this instead
/// ```
#[inline(always)]
#[allow(unsafe_code)]
#[must_use = "input must be consumed"]
pub fn input(slice: &[u8]) -> &Input {
    // Cast the slice reference to a pointer.
    let slice_ptr: *const [u8] = slice;
    // Cast the slice pointer to a `Input` pointer.
    //
    // The compiler allows this as the types are compatible. This cast is safe
    // as `Input` is a wrapper around [u8]. As with std::path::Path, `Input` is
    // not marked repr(transparent) or repr(C).
    let input_ptr = slice_ptr as *const Input;
    // Re-borrow the `Input` pointer as a `Input` reference.
    //
    // This is safe as the lifetime from the slice is carried from the slice
    // reference to the `Input` reference.
    unsafe { &*input_ptr }
}

/// `Input` is an immutable wrapper around bytes to be processed.
///
/// It can only be created via [`dangerous::input()`](crate::input()) as so to
/// clearly point out where user-generated / dangerous input is consumed.
///
/// It is used along with [`Reader`] to process the input.
///
/// # Formatting
///
/// `Input` implements both [`fmt::Debug`] and [`fmt::Display`] with support for
/// pretty printing. See [`InputDisplay`] for formatting options.
#[derive(Eq, PartialEq)]
#[must_use = "input must be consumed"]
pub struct Input([u8]);

impl Input {
    /// Returns the underlying byte slice length.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.as_dangerous().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    /// Returns `true` if the underlying byte slice for `parent` contains that
    /// of `self` in the same section of memory with no bounds out of range.
    pub fn is_within(&self, parent: &Input) -> bool {
        parent.inclusive_range(self).is_some()
    }

    /// Returns an [`InputDisplay`] for formatting.
    #[inline(always)]
    pub const fn display(&self) -> InputDisplay<'_> {
        InputDisplay::new(self)
    }

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    #[inline(always)]
    pub const fn as_dangerous(&self) -> &[u8] {
        &self.0
    }

    /// Returns the underlying byte slice if it is not empty.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the the input is empty.
    #[inline(always)]
    pub fn to_dangerous_non_empty(&self) -> Result<&[u8], ExpectedLength<'_>> {
        if self.is_empty() {
            Err(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                context: SealedContext {
                    input: self,
                    operation: "extract non-empty byte slice",
                },
            })
        } else {
            Ok(self.as_dangerous())
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the the input could never be valid UTF-8
    /// and [`ExpectedLength`] if a UTF-8 code point was cut short. This is
    /// useful when parsing potentially incomplete buffers.
    #[inline]
    pub fn to_dangerous_str<'i, E>(&'i self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        match str::from_utf8(self.as_dangerous()) {
            Ok(s) => Ok(s),
            Err(utf8_err) => match utf8_err.error_len() {
                None => {
                    let invalid = &self.as_dangerous()[utf8_err.valid_up_to()..];
                    // As the first byte in a UTF-8 code point encodes its
                    // length as shown below, and as we never reach this branch
                    // if the code point only spans one byte, we just count the
                    // leading ones of the first byte to work out the expected
                    // length.
                    //
                    // | format   | len | description         |
                    // | -------- | --- | ------------------- |
                    // | 0ZZZZZZZ | 1   | unreachable         |
                    // | 110YYYYY | 2   | valid               |
                    // | 1110XXXX | 3   | valid               |
                    // | 11110VVV | 4   | valid               |
                    // | 11110XXX | N/A | invalid/unreachable |
                    Err(E::from(ExpectedLength {
                        min: invalid[0].leading_ones() as usize,
                        max: None,
                        span: input(invalid),
                        context: SealedContext {
                            input: self,
                            operation: "decode utf-8 str",
                        },
                    }))
                }
                Some(error_len) => {
                    let bytes = self.as_dangerous();
                    let error_start = utf8_err.valid_up_to();
                    let error_end = error_start + error_len;
                    Err(E::from(ExpectedValid {
                        expected: "utf-8 code point",
                        found: "invalid value",
                        span: input(&bytes[error_start..error_end]),
                        context: SealedContext {
                            input: self,
                            operation: "decode utf-8 str",
                        },
                    }))
                }
            },
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short or the input is
    /// empty. This is useful when parsing potentially incomplete buffers.
    #[inline]
    pub fn to_dangerous_non_empty_str<'i, E>(&'i self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                context: SealedContext {
                    input: self,
                    operation: "decode non-empty utf-8 str",
                },
            }))
        } else {
            self.to_dangerous_str()
        }
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the function does, or there is trailing
    /// input.
    pub fn read_all<'i, F, O, E>(&'i self, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<O, E>,
        E: Error,
        E: From<ExpectedLength<'i>>,
    {
        let mut reader = Reader::new(self);
        f(&mut reader)
            .map_err(|err| {
                E::with_context(
                    err,
                    SealedContext {
                        input: self,
                        operation: "read all",
                    },
                )
            })
            .and_then(|ok| {
                if reader.at_end() {
                    Ok(ok)
                } else {
                    Err(E::from(ExpectedLength {
                        min: 0,
                        max: Some(0),
                        span: self,
                        context: SealedContext {
                            input: self,
                            operation: "read all",
                        },
                    }))
                }
            })
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the function does, or there is trailing
    /// input.
    pub fn read_partial<'i, F, O, E>(&'i self, f: F) -> Result<(O, &'i Input), E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<O, E>,
        E: Error,
    {
        let mut reader = Reader::new(self);
        f(&mut reader)
            .map(|ok| (ok, reader.take_all()))
            .map_err(|err| {
                E::with_context(
                    err,
                    SealedContext {
                        input: self,
                        operation: "read partial",
                    },
                )
            })
    }

    /// Returns the first byte in the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn first<'i, E>(&'i self) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.as_dangerous().first().copied().ok_or_else(|| {
            E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                context: SealedContext {
                    input: self,
                    operation: "extract first byte",
                },
            })
        })
    }

    /// Returns an empty `Input` pointing the end of `self`.
    #[inline(always)]
    pub(crate) fn end(&self) -> &Input {
        input(&self.as_dangerous()[self.len()..])
    }

    /// Splits the input into the first byte and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn split_first<'i, E>(&'i self) -> Result<(u8, &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(1)?;
        Ok((head.first()?, tail))
    }

    /// Splits the input into two at `mid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `mid > self.len()`.
    #[inline(always)]
    pub(crate) fn split_at<'i, E>(&'i self, mid: usize) -> Result<(&'i Input, &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        if mid > self.len() {
            Err(E::from(ExpectedLength {
                min: mid,
                max: None,
                span: self,
                context: SealedContext {
                    input: self,
                    operation: "split length",
                },
            }))
        } else {
            let (head, tail) = self.as_dangerous().split_at(mid);
            Ok((input(head), input(tail)))
        }
    }

    /// Splits the input into two at `max`.
    #[inline(always)]
    pub(crate) fn split_max(&self, max: usize) -> (&Input, &Input) {
        if max > self.len() {
            (self, self.end())
        } else {
            let (head, tail) = self.as_dangerous().split_at(max);
            (input(head), input(tail))
        }
    }

    #[inline(always)]
    pub(crate) fn split_sub(&self, sub: &Input) -> Option<(&Input, &Input)> {
        self.inclusive_range(sub).map(|range| {
            let bytes = self.as_dangerous();
            let head = &bytes[..range.start];
            let tail = &bytes[range.end..];
            (input(head), input(tail))
        })
    }

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<'i, F>(&'i self, mut f: F) -> (&'i Input, &'i Input)
    where
        F: FnMut(&'i Input, u8) -> bool,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let tail = input(tail);
            if !f(&tail, *byte) {
                return (input(head), tail);
            }
        }
        (self, self.end())
    }

    /// Trys to split the input while the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn try_split_while<'i, F, E>(&'i self, mut f: F) -> Result<(&'i Input, &'i Input), E>
    where
        F: FnMut(&'i Input, u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let tail = input(tail);
            if !f(&tail, *byte)? {
                return Ok((input(head), tail));
            }
        }
        Ok((self, self.end()))
    }

    #[inline(always)]
    pub(crate) fn inclusive_range(&self, sub: &Input) -> Option<Range<usize>> {
        let self_bounds = self.as_dangerous_ptr_range();
        let sub_bounds = sub.as_dangerous_ptr_range();
        if (self_bounds.start == sub_bounds.start || self_bounds.contains(&sub_bounds.start))
            && (self_bounds.end == sub_bounds.end || self_bounds.contains(&sub_bounds.end))
        {
            let start = sub_bounds.start as usize - self_bounds.start as usize;
            let end = start + sub.len();
            Some(start..end)
        } else {
            None
        }
    }

    // TODO: use https://github.com/rust-lang/rust/issues/65807 when stable
    fn as_dangerous_ptr_range(&self) -> Range<*const u8> {
        let bytes = self.as_dangerous();
        let start = bytes.as_ptr();
        // Note: will never wrap, but we are just escaping the use of unsafe
        let end = bytes.as_ptr().wrapping_add(bytes.len());
        start..end
    }
}

impl AsRef<Input> for Input {
    fn as_ref(&self) -> &Input {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Equality

impl PartialEq<[u8]> for Input {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl PartialEq<[u8]> for &Input {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl PartialEq<Input> for [u8] {
    fn eq(&self, other: &Input) -> bool {
        other.as_dangerous() == self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = InputDisplay::from_formatter(self, f);
        f.debug_tuple("Input").field(&display).finish()
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InputDisplay::from_formatter(self, f).fmt(f)
    }
}
