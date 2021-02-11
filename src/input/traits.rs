use core::convert::Infallible;

use crate::display::InputDisplay;
use crate::error::{
    with_context, CoreContext, CoreExpected, CoreOperation, ExpectedLength, ExpectedValid,
    ExpectedValue, External, Length, Value, WithChildContext, WithContext,
};
use crate::fmt::{Debug, Display, DisplayBase};
use crate::input::pattern::Pattern;
use crate::reader::Reader;

use super::{Bound, Bytes, MaybeString, Prefix, Span, String};

/// Implemented for immutable wrappers around bytes to be processed ([`Bytes`]/[`String`]).
///
/// It can only be created via [`dangerous::input()`] as so to clearly point out
/// where untrusted / dangerous input is consumed and takes the form of either
/// [`Bytes`] or [`String`].
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

    /// Decodes the underlying byte slice into a UTF-8 [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    fn into_string<E>(self) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>;

    /// Consumes `self` into [`MaybeString`] returning `MaybeString::String` if
    /// the underlying bytes are known `UTF-8`.
    fn into_maybe_string(self) -> MaybeString<'i>;

    /// Returns an [`InputDisplay`] for formatting.
    fn display(&self) -> InputDisplay<'i>;

    ///////////////////////////////////////////////////////////////////////////
    // Provided methods

    /// Returns the underlying byte slice length.
    #[must_use]
    #[inline(always)]
    fn byte_len(&self) -> usize {
        self.as_dangerous_bytes().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[must_use]
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.byte_len() == 0
    }

    /// Returns `true` if [`Self::bound()`] is [`Bound::Both`].
    #[must_use]
    #[inline(always)]
    fn is_bound(&self) -> bool {
        self.bound() == Bound::Both
    }

    #[inline(always)]
    fn span(&self) -> Span {
        Span::from(self.as_dangerous_bytes())
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    #[inline]
    fn read_all<F, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<T, E>,
        E: WithContext<'i>,
        E: From<ExpectedLength<'i>>,
    {
        let mut r = Reader::new(self.clone());
        match r.context(
            CoreContext::from_operation(CoreOperation::ReadAll, self.span()),
            f,
        ) {
            Ok(ok) if r.at_end() => Ok(ok),
            Ok(_) => Err(E::from(ExpectedLength {
                len: Length::Exactly(0),
                context: CoreContext {
                    span: r.take_remaining().span(),
                    operation: CoreOperation::ReadAll,
                    expected: CoreExpected::NoTrailingInput,
                },
                input: self.into_maybe_string(),
            })),
            Err(err) => Err(err),
        }
    }

    /// Create a reader to read a part of the input and return the rest.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided function does.
    #[inline]
    fn read_partial<F, T, E>(self, f: F) -> Result<(T, Self), E>
    where
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<T, E>,
        E: WithContext<'i>,
    {
        let mut r = Reader::new(self.clone());
        match r.context(
            CoreContext::from_operation(CoreOperation::ReadPartial, self.span()),
            f,
        ) {
            Ok(ok) => Ok((ok, r.take_remaining())),
            Err(err) => Err(err),
        }
    }

    /// Create a reader to read a part of the input and return the rest
    /// without any errors.
    #[inline]
    fn read_infallible<F, T>(self, f: F) -> (T, Self)
    where
        F: FnOnce(&mut Reader<'i, Infallible, Self>) -> T,
    {
        let mut r = Reader::new(self);
        let ok = f(&mut r);
        (ok, r.take_remaining())
    }

    /// Tries to convert `self` into `T` with [`External`] error support.
    ///
    /// **Note**: it is the callers responsibility to ensure all of the input is
    /// handled. If the function may only consume a partial amount of input, use
    /// [`Reader::try_expect_external()`] instead.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the provided function returns an
    /// [`External`] error.
    #[inline]
    fn into_external<F, T, E, Ex>(self, expected: &'static str, f: F) -> Result<T, E>
    where
        F: FnOnce(Self) -> Result<T, Ex>,
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        Ex: External<'i>,
    {
        f(self.clone()).map_err(|external| {
            self.map_external_error(external, expected, CoreOperation::IntoExternal)
        })
    }

    /// Returns `self` if it is not empty.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the input is empty.
    fn into_non_empty<E>(self) -> Result<Self, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                len: Length::AtLeast(1),

                context: CoreContext {
                    span: self.span(),
                    operation: CoreOperation::IntoNonEmpty,
                    expected: CoreExpected::NonEmpty,
                },
                input: self.into_maybe_string(),
            }))
        } else {
            Ok(self)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Private requirements for any `Input`

pub trait Private<'i>: Sized + Clone + DisplayBase + Debug + Display {
    /// Smallest unit that can be consumed.
    type Token: BytesLength + Copy + 'static;

    /// Iterator of tokens.
    ///
    /// Returns the byte index of the token and the token itself.
    type TokenIter: Iterator<Item = (usize, Self::Token)>;

    /// Returns an empty `Input` pointing the end of `self`.
    fn end(self) -> Self;

    /// Returns a token iterator.
    fn tokens(self) -> Self::TokenIter;

    // Return self with its end bound removed.
    fn into_unbound_end(self) -> Self;

    /// Verifies a token boundary.
    ///
    /// The start and end of the input (when `index == self.byte_len()`) are
    /// considered to be boundaries.
    ///
    /// Returns what was expected at that index.
    fn verify_token_boundary(&self, index: usize) -> Result<(), CoreExpected>;

    /// Split the input at the token index `mid`.
    ///
    /// Return `Some` if mid is a valid index, `None` if not.
    fn split_at_opt(self, mid: usize) -> Option<(Self, Self)>;

    /// Splits the input at the byte index `mid` without any validation.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that it is valid to split the structure at `mid`.
    unsafe fn split_at_byte_unchecked(self, mid: usize) -> (Self, Self);
}

///////////////////////////////////////////////////////////////////////////////
// Private extensions to any `Input`

pub(crate) trait PrivateExt<'i>: Input<'i> {
    /// Returns the underlying byte slice of the input.
    #[inline(always)]
    fn as_dangerous_bytes(&self) -> &'i [u8] {
        self.clone().into_bytes().as_dangerous()
    }

    /// Splits the input into two at the token index `mid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `mid > self.len()`.
    #[inline(always)]
    fn split_at<E>(self, mid: usize, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.clone().split_at_opt(mid).ok_or_else(|| {
            E::from(ExpectedLength {
                len: Length::AtLeast(mid),

                context: CoreContext {
                    span: self.span(),
                    operation,
                    expected: CoreExpected::EnoughInputFor("split"),
                },
                input: self.into_maybe_string(),
            })
        })
    }

    /// Splits the input into two at the byte index `mid`.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if `mid > self.len()` and [`ExpectedValid`]
    /// if `mid` is not a valid token boundary.
    #[inline(always)]
    fn split_at_byte<E>(self, mid: usize, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        if self.byte_len() < mid {
            Err(E::from(ExpectedLength {
                len: Length::AtLeast(mid),

                context: CoreContext {
                    span: self.span(),
                    operation,
                    expected: CoreExpected::EnoughInputFor("split"),
                },
                input: self.into_maybe_string(),
            }))
        } else {
            match self.verify_token_boundary(mid) {
                Ok(()) => {
                    // SAFETY: index `mid` has been checked to be a valid token
                    // boundary.
                    Ok(unsafe { self.split_at_byte_unchecked(mid) })
                }
                Err(expected) => Err(E::from(ExpectedValid {
                    #[cfg(feature = "retry")]
                    retry_requirement: None,
                    context: CoreContext {
                        span: self.as_dangerous_bytes()[mid..mid].into(),
                        operation,
                        expected,
                    },
                    input: self.into_maybe_string(),
                })),
            }
        }
    }

    /// Splits the input into the first token and whatever remains.
    #[inline(always)]
    fn split_token_opt(self) -> Option<(Self::Token, Self)> {
        self.clone().tokens().next().map(|(_, t)| {
            // SAFETY: ByteLength guarantees a correct implementation for
            // returning the length of a token. The token iterator returned a
            // token for us, so we know we can split it off safely.
            let (_, tail) = unsafe { self.split_at_byte_unchecked(t.byte_len()) };
            (t, tail)
        })
    }

    /// Splits the input into the first token and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    fn split_token<E>(self, operation: CoreOperation) -> Result<(Self::Token, Self), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.clone().split_token_opt().ok_or_else(|| {
            E::from(ExpectedLength {
                len: Length::AtLeast(1),

                context: CoreContext {
                    span: self.span(),
                    operation,
                    expected: CoreExpected::EnoughInputFor("token"),
                },
                input: self.into_maybe_string(),
            })
        })
    }

    /// Splits a prefix from the input if it is present.
    #[inline(always)]
    fn split_prefix_opt<P>(self, prefix: P) -> (Option<Self>, Self)
    where
        P: Prefix<Self>,
    {
        if prefix.is_prefix_of(&self) {
            // SAFETY: we just validated that prefix is within the input so its
            // length is a valid index.
            let (head, tail) = unsafe { self.split_at_byte_unchecked(prefix.byte_len()) };
            (Some(head), tail)
        } else {
            (None, self)
        }
    }

    /// Splits a prefix from the input if it is present.
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not have the prefix.
    #[inline(always)]
    fn split_prefix<P, E>(self, prefix: P, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Prefix<Self> + Into<Value<'i>>,
    {
        match self.clone().split_prefix_opt(&prefix) {
            (Some(head), tail) => Ok((head, tail)),
            (None, unmatched) => {
                let bytes = unmatched.as_dangerous_bytes();
                let prefix_len = prefix.byte_len();
                let actual = if bytes.len() > prefix_len {
                    &bytes[..prefix_len]
                } else {
                    &bytes
                };
                Err(E::from(ExpectedValue {
                    expected: prefix.into(),
                    context: CoreContext {
                        span: actual.into(),
                        operation,
                        expected: CoreExpected::ExactValue,
                    },
                    input: self.into_maybe_string(),
                }))
            }
        }
    }

    /// Splits at a pattern in the input if it is present.
    #[inline(always)]
    fn split_until_opt<P>(self, pattern: P) -> Option<(Self, Self)>
    where
        P: Pattern<Self>,
    {
        pattern.find_match(&self).map(|(index, _)| {
            // SAFETY: Pattern guarantees it returns valid indexes.
            unsafe { self.split_at_byte_unchecked(index) }
        })
    }

    /// Splits at a pattern in the input if it is present.
    #[inline(always)]
    fn split_until_consume_opt<P>(self, pattern: P) -> Option<(Self, Self)>
    where
        P: Pattern<Self>,
    {
        pattern.find_match(&self).map(|(index, len)| {
            // SAFETY: Pattern guarantees it returns valid indexes.
            let (head, tail) = unsafe { self.split_at_byte_unchecked(index) };
            let (_, tail) = unsafe { tail.split_at_byte_unchecked(len) };
            (head, tail)
        })
    }

    /// Splits the input up to when the pattern matches.
    #[inline(always)]
    fn split_until<P, E>(self, pattern: P, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<Self> + Into<Value<'i>> + Copy,
    {
        self.clone().split_until_opt(pattern).ok_or_else(|| {
            E::from(ExpectedValue {
                expected: pattern.into(),
                context: CoreContext {
                    span: self.span(),
                    operation,
                    expected: CoreExpected::PatternMatch,
                },
                input: self.into_maybe_string(),
            })
        })
    }

    /// Splits at a pattern in the input if it is present.
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not have the pattern.
    #[inline(always)]
    fn split_until_consume<P, E>(
        self,
        pattern: P,
        operation: CoreOperation,
    ) -> Result<(Self, Self), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<Self> + Into<Value<'i>> + Copy,
    {
        self.clone()
            .split_until_consume_opt(pattern)
            .ok_or_else(|| {
                E::from(ExpectedValue {
                    expected: pattern.into(),
                    context: CoreContext {
                        span: self.span(),
                        operation,
                        expected: CoreExpected::PatternMatch,
                    },
                    input: self.into_maybe_string(),
                })
            })
    }

    /// Splits the input up to when the provided function returns `false`.
    #[inline(always)]
    fn split_while_opt<P>(self, pattern: P) -> Option<(Self, Self)>
    where
        P: Pattern<Self>,
    {
        pattern.find_reject(&self).map(|i| {
            // SAFETY: Pattern guarantees it returns valid indexes.
            unsafe { self.split_at_byte_unchecked(i) }
        })
    }

    /// Tries to split the input up to when the provided function returns
    /// `false`.
    ///
    /// # Errors
    ///
    /// Returns an error from the provided function if it fails.
    #[inline(always)]
    fn try_split_while<F, E>(self, mut f: F, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: WithContext<'i>,
        F: FnMut(Self::Token) -> Result<bool, E>,
    {
        // For each token, lets make sure it matches the predicate.
        for (i, token) in self.clone().tokens() {
            // Check if the token doesn't match the predicate.
            let should_continue = with_context(
                CoreContext::from_operation(operation, self.span()),
                self.clone(),
                || f(token),
            )?;
            if !should_continue {
                // Split the input up to, but not including the token.
                // `i` derived from the token iterator is always a valid index
                // for the input.
                let (head, tail) = unsafe { self.split_at_byte_unchecked(i) };
                // Return the split input parts.
                return Ok((head, tail));
            }
        }
        Ok((self.clone(), self.end()))
    }

    /// Splits the input at what was read and what was remaining.
    #[inline(always)]
    fn split_consumed<F, E>(self, f: F) -> (Self, Self)
    where
        F: FnOnce(&mut Reader<'i, E, Self>),
    {
        let mut reader = Reader::new(self.clone());
        // Consume input.
        f(&mut reader);
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed.
        let mid = self.byte_len() - tail.byte_len();
        // SAFETY: we take mid as the difference between the parent slice and
        // the remaining slice left over from the reader. This means the index
        // can only ever be valid.
        let (head, _) = unsafe { self.split_at_byte_unchecked(mid) };
        // We derive the bound constraint from self. If the tail start is
        // undetermined this means the last bit of input consumed could be
        // longer if there was more available and as such makes the end of input
        // we return unbounded.
        if tail.bound() == Bound::None {
            (head.into_unbound_end(), tail)
        } else {
            (head, tail)
        }
    }

    /// Tries to split the input at what was read and what was remaining.
    ///
    /// # Errors
    ///
    /// Returns an error from the provided function if it fails.
    #[inline(always)]
    fn try_split_consumed<F, E>(self, f: F, operation: CoreOperation) -> Result<(Self, Self), E>
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self.clone());
        // Consume input.
        reader.context(CoreContext::from_operation(operation, self.span()), f)?;
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed.
        let mid = self.byte_len() - tail.byte_len();
        // SAFETY: we take mid as the difference between the parent slice and
        // the remaining slice left over from the reader. This means the index
        // can only ever be valid.
        let (head, _) = unsafe { self.split_at_byte_unchecked(mid) };
        // We derive the bound constraint from self. If the tail start is
        // undetermined this means the last bit of input consumed could be
        // longer if there was more available and as such makes the end of input
        // we return unbounded.
        if tail.bound() == Bound::None {
            Ok((head.into_unbound_end(), tail))
        } else {
            Ok((head, tail))
        }
    }

    /// Splits the input from the value expected to be read.
    ///
    /// # Errors
    ///
    /// Returns an error if the expected value was not present.
    #[inline(always)]
    fn split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: CoreOperation,
    ) -> Result<(T, Self), E>
    where
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Option<T>,
    {
        let mut reader = Reader::new(self.clone());
        if let Some(ok) = f(&mut reader) {
            Ok((ok, reader.take_remaining()))
        } else {
            let tail = reader.take_remaining();
            let span = self.as_dangerous_bytes()[..self.byte_len() - tail.byte_len()].into();
            Err(E::from(ExpectedValid {
                #[cfg(feature = "retry")]
                retry_requirement: None,
                context: CoreContext {
                    span,
                    expected: CoreExpected::Valid(expected),
                    operation,
                },
                input: self.into_maybe_string(),
            }))
        }
    }

    /// Tries to split the input from the value expected to be read.
    ///
    /// # Errors
    ///
    /// Returns an error from the provided function if it fails or if the
    /// expected value was not present.
    #[inline(always)]
    fn try_split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: CoreOperation,
    ) -> Result<(T, Self), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E, Self>) -> Result<Option<T>, E>,
    {
        let mut context = CoreContext {
            span: self.span(),
            expected: CoreExpected::Valid(expected),
            operation,
        };
        let mut reader = Reader::new(self.clone());
        match reader.context(context, f) {
            Ok(Some(ok)) => Ok((ok, reader.take_remaining())),
            Ok(None) => {
                let tail = reader.take_remaining();
                context.span =
                    self.as_dangerous_bytes()[..self.byte_len() - tail.byte_len()].into();
                Err(E::from(ExpectedValid {
                    #[cfg(feature = "retry")]
                    retry_requirement: None,
                    context,
                    input: self.into_maybe_string(),
                }))
            }
            Err(err) => Err(err),
        }
    }

    /// Tries to split the input from a value expected with support for an
    /// external error.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if:
    ///
    /// - the provided function returns an amount of input read not aligned to a
    ///   token boundary
    /// - the provided function returns an [`External`] error.
    ///
    /// Returns [`ExpectedLength`] if:
    ///
    /// - the provided function returns an amount of input read that is greater
    ///   than the actual length
    #[inline(always)]
    fn try_split_expect_external<F, T, E, Ex>(
        self,
        f: F,
        expected: &'static str,
        operation: CoreOperation,
    ) -> Result<(T, Self), E>
    where
        F: FnOnce(Self) -> Result<(T, usize), Ex>,
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        Ex: External<'i>,
    {
        match f(self.clone()) {
            Ok((ok, read)) => self
                .split_at_byte(read, operation)
                .map(|(_, remaining)| (ok, remaining)),
            Err(external) => Err(self.map_external_error(external, expected, operation)),
        }
    }

    fn map_external_error<E, Ex>(
        self,
        external: Ex,
        expected: &'static str,
        operation: CoreOperation,
    ) -> E
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        Ex: External<'i>,
    {
        let error = E::from(ExpectedValid {
            #[cfg(feature = "retry")]
            retry_requirement: external.retry_requirement(),
            context: CoreContext {
                span: external.span().unwrap_or_else(|| self.span()),
                expected: CoreExpected::Valid(expected),
                operation,
            },
            input: self.into_maybe_string(),
        });
        external
            .push_backtrace(WithChildContext::new(error))
            .unwrap()
    }
}

impl<'i, T> PrivateExt<'i> for T where T: Input<'i> {}

///////////////////////////////////////////////////////////////////////////////
// BytesLength

pub unsafe trait BytesLength: Copy {
    fn byte_len(self) -> usize;
}

unsafe impl<T> BytesLength for &T
where
    T: BytesLength,
{
    #[inline(always)]
    fn byte_len(self) -> usize {
        (*self).byte_len()
    }
}

unsafe impl BytesLength for u8 {
    #[inline(always)]
    fn byte_len(self) -> usize {
        1
    }
}

unsafe impl BytesLength for char {
    #[inline(always)]
    fn byte_len(self) -> usize {
        self.len_utf8()
    }
}

unsafe impl BytesLength for &[u8] {
    #[inline(always)]
    fn byte_len(self) -> usize {
        self.len()
    }
}

unsafe impl BytesLength for &str {
    #[inline(always)]
    fn byte_len(self) -> usize {
        self.as_bytes().len()
    }
}

macro_rules! impl_array_bytes_len {
    ($($n:expr),*) => {
        $(
            unsafe impl BytesLength for &[u8; $n] {
                #[inline(always)]
                fn byte_len(self) -> usize {
                    self.len()
                }
            }
        )*
    };
}

for_common_array_sizes!(impl_array_bytes_len);

///////////////////////////////////////////////////////////////////////////////
// IntoInput

pub trait IntoInput<'i>: Copy {
    type Input: Input<'i>;

    fn into_input(self) -> Self::Input;
}

impl<'i, T> IntoInput<'i> for &T
where
    T: IntoInput<'i>,
{
    type Input = T::Input;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        (*self).into_input()
    }
}

impl<'i> IntoInput<'i> for &'i [u8] {
    type Input = Bytes<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}

impl<'i> IntoInput<'i> for &'i str {
    type Input = String<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        String::new(self, Bound::Start)
    }
}

macro_rules! impl_array_into_input {
    ($($n:expr),*) => {
        $(
            impl<'i> IntoInput<'i> for &'i [u8; $n] {
                type Input = Bytes<'i>;

                #[inline(always)]
                fn into_input(self) -> Self::Input {
                    Bytes::new(self, Bound::Start)
                }
            }
        )*
    };
}

for_common_array_sizes!(impl_array_into_input);
