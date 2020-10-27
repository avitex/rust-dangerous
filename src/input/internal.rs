use core::convert::TryInto;

use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, ExpectedValue, FromContext,
    OperationContext, ToRetryRequirement,
};
use crate::reader::Reader;
use crate::util::byte;

use super::{input, Input};

// All functions defined in internal are used within other functions that expose
// public functionality.
//
// All functionality in this module:
// - Will never panic
// - Will use a provided operation name for any initial error
// - Will not add additional context to the initial error if any
// - Will wrap a provided function's error with the provided operation name
// - Will be inlined into the callee

impl<'i> Input<'i> {
    #[inline(always)]
    #[cfg(not(feature = "no-input-bound"))]
    pub(crate) const fn new(bytes: &'i [u8], bound: bool) -> Self {
        Self { bytes, bound }
    }

    #[inline(always)]
    #[cfg(feature = "no-input-bound")]
    pub(crate) const fn new(bytes: &'i [u8], _bound: bool) -> Self {
        Self { bytes }
    }

    #[inline(always)]
    #[allow(clippy::unused_self)]
    #[cfg(feature = "no-input-bound")]
    pub(crate) fn is_bound(&self) -> bool {
        false
    }

    #[inline(always)]
    #[cfg(not(feature = "no-input-bound"))]
    pub(crate) fn is_bound(&self) -> bool {
        self.bound
    }

    /// Returns an empty `Input` pointing the end of `self`.
    #[inline(always)]
    pub(crate) fn end(&self) -> Input<'i> {
        input(&self.as_dangerous()[self.len()..])
    }

    #[inline(always)]
    pub(crate) fn has_prefix(&self, prefix: &[u8]) -> bool {
        if self.len() >= prefix.len() {
            let bytes = self.as_dangerous();
            let (head, _) = bytes.split_at(prefix.len());
            prefix == head
        } else {
            false
        }
    }
}

impl<'i> Input<'i> {
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
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "a byte",
                },
            })
        })
    }

    #[inline(always)]
    pub(crate) fn split_prefix<E>(
        self,
        prefix: &'i [u8],
        operation: &'static str,
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        let (maybe_prefix, tail) = self.clone().split_max(prefix.len());
        if maybe_prefix == *prefix {
            Ok((maybe_prefix, tail))
        } else {
            Err(E::from(ExpectedValue {
                actual: maybe_prefix.as_dangerous(),
                expected: prefix,
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "exact value",
                },
            }))
        }
    }

    #[inline(always)]
    pub(crate) fn split_prefix_u8<E>(
        self,
        prefix: u8,
        operation: &'static str,
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        self.split_prefix(byte::to_slice(prefix), operation)
    }

    /// Splits the input into the first byte and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn split_first<E>(self, operation: &'static str) -> Result<(u8, Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(1, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous()[0], tail)),
            Err(err) => Err(err),
        }
    }

    /// Splits the input into two at `mid`.
    #[inline(always)]
    pub(crate) fn split_at_opt(self, mid: usize) -> Option<(Input<'i>, Input<'i>)> {
        if mid > self.len() {
            None
        } else {
            let (head, tail) = self.as_dangerous().split_at(mid);
            Some((Input::new(head, true), Input::new(tail, self.is_bound())))
        }
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
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.clone().split_at_opt(mid).ok_or_else(|| {
            E::from(ExpectedLength {
                min: mid,
                max: None,
                span: self.as_dangerous(),
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "enough input",
                },
            })
        })
    }

    #[inline(always)]
    pub(crate) fn split_consumed<F, E>(self, mut f: F) -> (Input<'i>, Input<'i>)
    where
        E: FromContext<'i>,
        F: FnMut(&mut Reader<'i, E>),
    {
        let mut reader = Reader::new(self.clone());
        f(&mut reader);
        let bytes = self.as_dangerous();
        let tail = reader.take_remaining();
        let head = &bytes[..bytes.len() - tail.len()];
        (Input::new(head, self.is_bound()), tail)
    }

    #[inline(always)]
    pub(crate) fn split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Input<'i>), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Option<T>,
    {
        let mut reader = Reader::new(self.clone());
        match f(&mut reader) {
            Some(ok) => Ok((ok, reader.take_remaining())),
            None => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span,
                    input: self,
                    context: ExpectedContext {
                        expected,
                        operation,
                    },
                    retry_requirement: None,
                }))
            }
        }
    }

    #[inline(always)]
    pub(crate) fn try_split_expect<F, T, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Input<'i>), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Result<Option<T>, E>,
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
                    input: self,
                    context,
                    retry_requirement: None,
                }))
            }
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn try_split_expect_erased<F, T, R, E>(
        self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, Input<'i>), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Result<T, R>,
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
                    input: self,
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
    pub(crate) fn try_split_consumed<F, E>(
        self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: FromContext<'i>,
        F: FnMut(&mut Reader<'i, E>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self.clone());
        with_context(self.clone(), OperationContext(operation), || f(&mut reader))?;
        let tail = reader.take_remaining();
        let head = &self.as_dangerous()[..self.len() - tail.len()];
        Ok((Input::new(head, self.is_bound()), tail))
    }

    /// Splits the input into two at `max`.
    #[inline(always)]
    pub(crate) fn split_max(self, max: usize) -> (Input<'i>, Input<'i>) {
        if max > self.len() {
            (self.clone(), self.end())
        } else {
            let (head, tail) = self.as_dangerous().split_at(max);
            (Input::new(head, true), Input::new(tail, self.is_bound()))
        }
    }

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<F>(self, mut f: F) -> (Input<'i>, Input<'i>)
    where
        F: FnMut(u8) -> bool,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let should_continue = f(*byte);
            if !should_continue {
                return (
                    Input::new(head, self.is_bound()),
                    Input::new(tail, self.is_bound()),
                );
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
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: FromContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let should_continue =
                with_context(self.clone(), OperationContext(operation), || f(*byte))?;
            if !should_continue {
                return Ok((
                    Input::new(head, self.is_bound()),
                    Input::new(tail, self.is_bound()),
                ));
            }
        }
        Ok((self.clone(), self.end()))
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: replace with const generics when stable

    #[inline(always)]
    pub(crate) fn split_arr_1<E>(self, operation: &'static str) -> Result<([u8; 1], Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_first(operation) {
            Ok((byte, tail)) => Ok(([byte], tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_2<E>(self, operation: &'static str) -> Result<([u8; 2], Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(2, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_4<E>(self, operation: &'static str) -> Result<([u8; 4], Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(4, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_8<E>(self, operation: &'static str) -> Result<([u8; 8], Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(8, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    pub(crate) fn split_arr_16<E>(self, operation: &'static str) -> Result<([u8; 16], Input<'i>), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.split_at(16, operation) {
            Ok((head, tail)) => Ok((head.as_dangerous().try_into().unwrap(), tail)),
            Err(err) => Err(err),
        }
    }
}
