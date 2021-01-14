use core::ops::Range;

use crate::display::InputDisplay;
use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, ToRetryRequirement, WithContext,
};
use crate::fmt::{Debug, Display, DisplayBase};
use crate::reader::Reader;
use crate::util::slice;

use super::{Bound, Bytes, MaybeString};

/// An [`Input`] is an immutable wrapper around bytes to be processed.
///
/// It can only be created via [`dangerous::input()`] as so to clearly point out
/// where untrusted / dangerous input is consumed.
///
/// It is used along with [`Reader`] to process the input.
///
/// # Formatting
///
/// `Input` implements support for pretty printing. See [`InputDisplay`] for
/// formatting options.
///
/// [`dangerous::input()`]: crate::input()
#[must_use = "input must be consumed"]
pub trait Input<'i>: Private<'i> {
    #[must_use]
    fn bound(&self) -> Bound;

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    fn as_dangerous(&self) -> &'i [u8];

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// If the underlying byte slice is known to be valid UTF-8 this is will a
    /// cheap operation, otherwise the bytes will be validated.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input is not valid UTF-8.
    fn to_dangerous_str<E>(&self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>;

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
    ///     .into_bound()
    ///     .read_partial(|r| r.take(5))
    ///     .unwrap_err();
    ///
    /// // If the input was not bound, this wouldn't be fatal.
    /// assert!(error.is_fatal());
    /// ```
    ///
    /// [`RetryRequirement`]: crate::error::RetryRequirement
    fn into_bound(self) -> Self;

    fn into_bytes(self) -> Bytes<'i>;

    fn into_maybe_string(self) -> MaybeString<'i>;

    /// Returns an [`InputDisplay`] for formatting.
    fn display(&self) -> InputDisplay<'i>;

    ///////////////////////////////////////////////////////////////////////////
    // Provided methods

    /// Returns the underlying byte slice length.
    #[inline(always)]
    #[must_use]
    fn len(&self) -> usize {
        self.as_dangerous().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[inline(always)]
    #[must_use]
    fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    #[must_use]
    fn is_bound(&self) -> bool {
        self.bound() == Bound::Both
    }

    /// Returns `true` if the underlying byte slice for `parent` contains that
    /// of `self` in the same section of memory with no bounds out of range.
    #[must_use]
    fn is_within<'p>(&self, parent: &impl Input<'p>) -> bool {
        slice::is_sub_slice(parent.as_dangerous(), self.as_dangerous())
    }

    /// Returns the occurrences of `needle` within the underlying byte slice.
    ///
    /// It is recommended to enable the `bytecount` dependency when using this
    /// function for better performance.
    #[must_use]
    fn count(&self, needle: u8) -> usize {
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
    fn span_of<'p>(&self, parent: &impl Input<'p>) -> Option<Range<usize>> {
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
    fn span_of_non_empty<'p>(&self, parent: &impl Input<'p>) -> Option<Range<usize>> {
        if self.is_empty() {
            None
        } else {
            self.span_of(parent)
        }
    }

    /// Returns the underlying byte slice if it is not empty.
    ///
    /// See [`Input::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty.
    fn to_dangerous_non_empty<E>(&self) -> Result<&'i [u8], E>
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
    /// See [`Input::as_dangerous`] for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty or [`ExpectedValid`] if
    /// the input is not valid UTF-8.
    fn to_dangerous_non_empty_str<E>(&self) -> Result<&'i str, E>
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

///////////////////////////////////////////////////////////////////////////////
// Private requirements for any `Input`

pub trait Private<'i>: Sized + Clone + DisplayBase + Debug + Display {
    /// Returns an empty `Input` pointing the end of `self`.
    fn end(self) -> Self;

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)>;

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)>;
}

///////////////////////////////////////////////////////////////////////////////
// Private extensions to any `Input`

pub(crate) trait PrivateExt<'i>: Input<'i> {
    fn split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Self), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Option<T>,
    {
        self.try_split_expect(|r| Ok(f(r)), expected, operation)
    }

    fn try_split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Self), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<Option<T>, E>,
    {
        let context = ExpectedContext {
            expected,
            operation,
        };
        let mut reader = Reader::new(self.clone());
        match with_context(self.clone(), context, || f(&mut reader)) {
            Ok(Some(ok)) => Ok((ok, reader.take_remaining())),
            Ok(None) => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span,
                    input: self.into_maybe_string(),
                    context,
                    retry_requirement: None,
                }))
            }
            Err(err) => Err(err),
        }
    }

    fn try_split_expect_erased<F, T, R, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Self), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<T, R>,
        R: ToRetryRequirement,
    {
        let mut reader = Reader::new(self.clone());
        match f(&mut reader) {
            Ok(ok) => Ok((ok, reader.take_remaining())),
            Err(err) => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span,
                    input: self.into_maybe_string(),
                    context: ExpectedContext {
                        expected,
                        operation,
                    },
                    retry_requirement: err.to_retry_requirement(),
                }))
            }
        }
    }
}

impl<'i, T> PrivateExt<'i> for T where T: Input<'i> {}
