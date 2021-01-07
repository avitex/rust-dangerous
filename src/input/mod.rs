mod flags;
mod internal;

use core::convert::Infallible;
use core::ops::Range;
use core::{fmt, str};

use crate::display::InputDisplay;
use crate::error::{ExpectedContext, ExpectedLength, ExpectedValid, OperationContext, WithContext};
use crate::reader::Reader;
use crate::util::{slice, utf8};

use self::flags::Flags;

/// Creates a new `Input` from a byte slice.
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
#[must_use = "input must be consumed"]
pub const fn input(bytes: &[u8]) -> Input<'_> {
    Input::new(bytes, false)
}

/// `Input` is an immutable wrapper around bytes to be processed.
///
/// It can only be created via [`dangerous::input()`] as so to clearly point out
/// where untrusted / dangerous input is consumed.
///
/// It is used along with [`Reader`] to process the input.
///
/// # Formatting
///
/// `Input` implements both [`fmt::Debug`] and [`fmt::Display`] with support for
/// pretty printing. See [`InputDisplay`] for formatting options.
///
/// [`dangerous::input()`]: crate::input()
#[must_use = "input must be consumed"]
pub struct Input<'i> {
    bytes: &'i [u8],
    flags: Flags,
}

impl<'i> Input<'i> {
    /// Returns the underlying byte slice length.
    #[inline(always)]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.as_dangerous().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[inline(always)]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    /// Returns `true` if the underlying byte slice is bound.
    ///
    /// See [`Input::bound()`] for more documentation.
    #[inline(always)]
    #[must_use]
    pub const fn is_bound(&self) -> bool {
        self.flags.is_bound()
    }

    /// Returns `true` if the underlying byte slice for `parent` contains that
    /// of `self` in the same section of memory with no bounds out of range.
    #[must_use]
    pub fn is_within(&self, parent: &Input<'_>) -> bool {
        slice::is_sub_slice(parent.as_dangerous(), self.as_dangerous())
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

    /// Returns `self` as a bound `Input`.
    ///
    /// Bound `Input` carries the guarantee that it will not be extended in
    /// future passes and as a result will not produce [`RetryRequirement`]s.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Invalid, ToRetryRequirement};
    ///
    /// let error: Invalid = dangerous::input(b"1234")
    ///     .bound()
    ///     .read_partial(|r| r.take(5))
    ///     .unwrap_err();
    ///
    /// // If the input was not bound, this wouldn't be fatal.
    /// assert!(error.is_fatal());
    /// ```
    ///
    /// [`RetryRequirement`]: crate::error::RetryRequirement
    pub fn bound(self) -> Self {
        Input::new(self.as_dangerous(), true)
    }

    /// Returns `Some(Range)` with the `start` and `end` offsets of `self`
    /// within the `parent`. `None` is returned if `self` is not within in the
    /// `parent`.
    ///
    /// # Example
    ///
    /// ```
    /// let parent = dangerous::input(&[1, 2, 3, 4]);
    /// let sub_range = 1..2;
    /// let sub = dangerous::input(&parent.as_dangerous()[sub_range.clone()]);
    ///
    /// assert_eq!(sub.span_of(&parent), Some(sub_range))
    /// ```
    #[must_use]
    pub fn span_of(&self, parent: &Input<'_>) -> Option<Range<usize>> {
        if self.is_within(parent) {
            let parent_bounds = parent.as_dangerous().as_ptr_range();
            let sub_bounds = self.as_dangerous().as_ptr_range();
            let start_offset = sub_bounds.start as usize - parent_bounds.start as usize;
            let end_offset = sub_bounds.end as usize - parent_bounds.start as usize;
            Some(start_offset..end_offset)
        } else {
            None
        }
    }

    /// Returns `Some(Range)` with the `start` and `end` offsets of `self`
    /// within the `parent`. `None` is returned if `self` is not within in the
    /// `parent` or `self` is empty.
    #[must_use]
    pub fn non_empty_span_of(&self, parent: &Input<'_>) -> Option<Range<usize>> {
        if self.is_empty() {
            None
        } else {
            self.span_of(parent)
        }
    }

    /// Returns an [`InputDisplay`] for formatting.
    #[inline(always)]
    pub fn display(&self) -> InputDisplay<'i> {
        InputDisplay::new(self)
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    pub fn read_all<F, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<T, E>,
        E: WithContext<'i>,
        E: From<ExpectedLength<'i>>,
    {
        let mut r = Reader::new(self.clone());
        match r.context(OperationContext("read all"), f) {
            Ok(ok) if r.at_end() => Ok(ok),
            Ok(_) => Err(E::from(ExpectedLength {
                min: 0,
                max: Some(0),
                span: r.take_remaining().as_dangerous(),
                input: self,
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
    pub fn read_partial<F, T, E>(self, f: F) -> Result<(T, Input<'i>), E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<T, E>,
        E: WithContext<'i>,
    {
        let mut r = Reader::new(self);
        match r.context(OperationContext("read partial"), f) {
            Ok(ok) => Ok((ok, r.take_remaining())),
            Err(err) => Err(err),
        }
    }

    /// Create a reader to read a part of the input and return the rest
    /// without any errors.
    pub fn read_infallible<F, T>(self, f: F) -> (T, Input<'i>)
    where
        F: FnOnce(&mut Reader<'i, Infallible>) -> T,
    {
        let mut r = Reader::new(self);
        let ok = f(&mut r);
        (ok, r.take_remaining())
    }

    ///////////////////////////////////////////////////////////////////////////
    // AsDangerous

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    #[inline(always)]
    #[must_use]
    pub const fn as_dangerous(&self) -> &'i [u8] {
        &self.bytes
    }

    /// Returns the underlying byte slice if it is not empty.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns a **non-retryable** [`ExpectedValid`] if the input is empty.
    pub fn to_dangerous_non_empty<E>(&self) -> Result<&'i [u8], E>
    where
        E: From<ExpectedValid<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedValid {
                retry_requirement: None,
                span: self.as_dangerous(),
                input: self.clone(),
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
    /// See `as_dangerous` for naming.
    ///
    /// If the underlying byte slice is known to be valid UTF-8 this is will a
    /// cheap operation, otherwise the bytes will be validated.
    ///
    /// # Errors
    ///
    /// Returns a **non-retryable** [`ExpectedValid`] if the input is not valid
    /// UTF-8.
    pub fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
    {
        let bytes = self.as_dangerous();
        if self.is_str() {
            // SAFETY: `is_str` guarantees that the bytes are valid UTF-8.
            unsafe { Ok(utf8::from_unchecked(bytes)) }
        } else {
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
                        input: self.clone(),
                        context: ExpectedContext {
                            operation: "convert input to str",
                            expected: "utf-8 code point",
                        },
                        retry_requirement: None,
                    }))
                }
            }
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns a **non-retryable** [`ExpectedValid`] if the input is either
    /// empty or not valid UTF-8.
    pub fn to_dangerous_non_empty_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedValid {
                retry_requirement: None,
                span: self.as_dangerous(),
                input: self.clone(),
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

///////////////////////////////////////////////////////////////////////////////
// Equality

impl<'i> PartialEq for Input<'i> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_dangerous() == other.as_dangerous()
    }
}

impl<'i> PartialEq<[u8]> for Input<'i> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<[u8]> for &Input<'i> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i> PartialEq<&[u8]> for Input<'i> {
    #[inline(always)]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_dangerous() == *other
    }
}

impl<'i> PartialEq<Input<'i>> for [u8] {
    #[inline(always)]
    fn eq(&self, other: &Input<'i>) -> bool {
        self == other.as_dangerous()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i> fmt::Debug for Input<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = if self.is_str() {
            InputDisplay::from_formatter(self, f).str_hint(true)
        } else {
            InputDisplay::from_formatter(self, f)
        };
        f.debug_tuple("Input").field(&display).finish()
    }
}

impl<'i> fmt::Display for Input<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = if self.is_str() {
            InputDisplay::from_formatter(self, f).str_hint(true)
        } else {
            InputDisplay::from_formatter(self, f)
        };
        display.fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Clone

impl<'i> Clone for Input<'i> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes,
            flags: self.flags,
        }
    }
}
