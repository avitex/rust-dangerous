use core::convert::Infallible;
use core::ops::Range;

use crate::display::InputDisplay;
#[cfg(feature = "retry")]
use crate::error::ToRetryRequirement;
use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, OperationContext, WithContext,
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
    /// Returns the [`Input`] [`Bound`].
    fn bound(&self) -> Bound;

    /// Returns `self` as a bound `Input`.
    ///
    /// Bound `Input` carries the guarantee that it will not be extended in
    /// future passes and as a result will not produce [`RetryRequirement`]s.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Invalid, ToRetryRequirement};
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

    /// Consumes `self` into [`Bytes`].
    fn into_bytes(self) -> Bytes<'i>;

    /// Consumes `self` into [`MaybeString`] returning `MaybeString::String` if
    /// the underlying bytes are known `UTF-8`.
    fn into_maybe_string(self) -> MaybeString<'i>;

    /// Returns an [`InputDisplay`] for formatting.
    fn display(&self) -> InputDisplay<'i>;

    ///////////////////////////////////////////////////////////////////////////
    // Provided methods

    /// Returns the underlying byte slice length.
    #[inline(always)]
    #[must_use]
    fn byte_len(&self) -> usize {
        self.as_dangerous_bytes().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[inline(always)]
    #[must_use]
    fn is_empty(&self) -> bool {
        self.byte_len() == 0
    }

    /// Returns `true` if [`Self::bound()`] is [`Bound::Both`].
    #[must_use]
    fn is_bound(&self) -> bool {
        self.bound() == Bound::Both
    }

    /// Returns `true` if the underlying byte slice for `parent` contains that
    /// of `self` in the same section of memory with no bounds out of range.
    #[must_use]
    fn is_within<'p>(&self, parent: &impl Input<'p>) -> bool {
        slice::is_sub_slice(parent.as_dangerous_bytes(), self.as_dangerous_bytes())
    }

    /// Returns `Some(Range)` with the `start` and `end` offsets of `self`
    /// within the `parent`. `None` is returned if `self` is not within in the
    /// `parent`.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let parent = dangerous::input(&[1, 2, 3, 4]);
    /// let sub_range = 1..2;
    /// let sub = dangerous::input(&parent.as_dangerous()[sub_range.clone()]);
    ///
    /// assert_eq!(sub.span_of(&parent), Some(sub_range))
    /// ```
    #[must_use]
    fn span_of<'p>(&self, parent: &impl Input<'p>) -> Option<Range<usize>> {
        if self.is_within(parent) {
            let parent_bounds = parent.as_dangerous_bytes().as_ptr_range();
            let sub_bounds = self.as_dangerous_bytes().as_ptr_range();
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

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    fn read_all<F, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<T, E>,
        E: WithContext<'i>,
        E: From<ExpectedLength<'i>>,
    {
        let mut r = Reader::new(self.clone());
        match r.context(OperationContext("read all"), f) {
            Ok(ok) if r.at_end() => Ok(ok),
            Ok(_) => Err(E::from(ExpectedLength {
                min: 0,
                max: Some(0),
                span: r.take_remaining().as_dangerous_bytes(),
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
    fn read_partial<F, T, E>(self, f: F) -> Result<(T, Self), E>
    where
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<T, E>,
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
    fn read_infallible<F, T>(self, f: F) -> (T, Self)
    where
        F: FnOnce(&mut Reader<'i, Infallible, Self>) -> T,
    {
        let mut r = Reader::new(self);
        let ok = f(&mut r);
        (ok, r.take_remaining())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Private requirements for any `Input`

pub trait Private<'i>: Sized + Clone + DisplayBase + Debug + Display {
    /// Returns an empty `Input` pointing the end of `self`.
    fn end(self) -> Self;

    fn into_unbound(self) -> Self;

    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)>;

    fn split_bytes_at_opt(self, mid: usize) -> Option<(Bytes<'i>, Bytes<'i>)>;

    /// Splits at a byte index without any validation.
    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self);
}

///////////////////////////////////////////////////////////////////////////////
// Private extensions to any `Input`

pub(crate) trait PrivateExt<'i>: Input<'i> {
    fn as_dangerous_bytes(&self) -> &'i [u8] {
        self.clone().into_bytes().as_dangerous()
    }

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
                let span = &self.as_dangerous_bytes()[..self.byte_len() - tail.byte_len()];
                Err(E::from(ExpectedValid {
                    span,
                    input: self.into_maybe_string(),
                    context,
                    #[cfg(feature = "retry")]
                    retry_requirement: None,
                }))
            }
            Err(err) => Err(err),
        }
    }

    // TODO: `not(feature = "retry")` support
    #[cfg(feature = "retry")]
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
                let span = &self.as_dangerous_bytes()[..self.byte_len() - tail.byte_len()];
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

    #[inline(always)]
    fn split_consumed<F, E>(self, f: F) -> (Self, Self)
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Reader<'i, E, Self>),
    {
        let mut reader = Reader::new(self.clone());
        f(&mut reader);
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed.
        let mid = self.byte_len() - tail.byte_len();
        let (head, _) = unsafe { self.split_at_byte_unchecked(mid) };
        // We derive the bound constraint from self. If the tail start is
        // undetermined this means the last bit of input consumed could be
        // longer if there was more available and as such makes the input we
        // return unbounded.
        if tail.bound() == Bound::None {
            (head.into_unbound(), tail)
        } else {
            (head, tail)
        }
    }

    #[inline(always)]
    fn try_split_consumed<F, E>(self, f: F, operation: &'static str) -> Result<(Self, Self), E>
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self.clone());
        with_context(self.clone(), OperationContext(operation), || f(&mut reader))?;
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed.
        let mid = self.byte_len() - tail.byte_len();
        let (head, _) = unsafe { self.split_at_byte_unchecked(mid) };
        // We derive the bound constraint from self. If the tail start is
        // undetermined this means the last bit of input consumed could be
        // longer if there was more available and as such makes the input we
        // return unbounded.
        if tail.bound() == Bound::None {
            Ok((head.into_unbound(), tail))
        } else {
            Ok((head, tail))
        }
    }
}

impl<'i, T> PrivateExt<'i> for T where T: Input<'i> {}
