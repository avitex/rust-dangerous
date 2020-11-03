use core::convert::TryInto;

use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, ExpectedValue, FromContext,
    OperationContext, ToRetryRequirement,
};
use crate::reader::Reader;
use crate::util::{byte, slice};

use super::{Flags, Input};

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
    pub(crate) const fn new(bytes: &'i [u8], bound: bool) -> Self {
        Self {
            bytes,
            flags: Flags::new(bound, false),
        }
    }

    #[inline(always)]
    pub(crate) const fn new_str(s: &'i str, bound: bool) -> Self {
        Self {
            bytes: s.as_bytes(),
            flags: Flags::new(bound, true),
        }
    }

    #[inline(always)]
    pub(crate) const fn is_str(&self) -> bool {
        self.flags.is_str()
    }

    #[inline(always)]
    pub(crate) fn has_prefix(&self, prefix: &[u8]) -> bool {
        self.as_dangerous().starts_with(prefix)
    }

    /// Returns an empty `Input` pointing the end of `self`.
    #[inline(always)]
    pub(crate) fn end(self) -> Input<'i> {
        Input::new(slice::end(self.as_dangerous()), self.is_bound())
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
        let actual = if let Some((head, tail)) = self.clone().split_at_opt(prefix.len()) {
            if head == prefix {
                return Ok((head, tail));
            } else {
                head.as_dangerous()
            }
        } else {
            self.as_dangerous()
        };
        Err(E::from(ExpectedValue {
            actual,
            expected: prefix,
            input: self,
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
    ) -> Result<(Input<'i>, Input<'i>), E>
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
            expected: byte::to_slice(prefix),
            input: self,
            context: ExpectedContext {
                operation,
                expected: "exact value",
            },
        }))
    }

    #[inline(always)]
    pub(crate) fn split_prefix_opt(self, prefix: &[u8]) -> (Option<Input<'i>>, Input<'i>) {
        if let Some((head, tail)) = self.clone().split_at_opt(prefix.len()) {
            if head == prefix {
                return (Some(head), tail);
            }
        }
        (None, self)
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

    /// Splits the input into two at `mid`.
    #[inline(always)]
    pub(crate) fn split_at_opt(self, mid: usize) -> Option<(Input<'i>, Input<'i>)> {
        slice::split_at_opt(self.as_dangerous(), mid).map(|(head, tail)| {
            // We split at a known length making the head input bound.
            let head = Input::new(head, true);
            // For the tail we derive the bound constraint from self.
            let tail = Input::new(tail, self.is_bound());
            // Return the split input parts.
            (head, tail)
        })
    }

    #[inline(always)]
    pub(crate) fn split_consumed<F, E>(self, f: F) -> (Input<'i>, Input<'i>)
    where
        E: FromContext<'i>,
        F: FnOnce(&mut Reader<'i, E>),
    {
        let mut reader = Reader::new(self.clone());
        f(&mut reader);
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed and derive the bound
        // constraint from self.
        // FIXME: is the fact this could be unbound a footgun?
        let head = Input::new(
            &self.as_dangerous()[..self.len() - tail.len()],
            self.is_bound(),
        );
        // Return the split input parts.
        (head, tail)
    }

    #[inline(always)]
    pub(crate) fn try_split_consumed<F, E>(
        self,
        f: F,
        operation: &'static str,
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: FromContext<'i>,
        F: FnOnce(&mut Reader<'i, E>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self.clone());
        with_context(self.clone(), OperationContext(operation), || f(&mut reader))?;
        // We take the remaining input.
        let tail = reader.take_remaining();
        // For the head, we take what we consumed and derive the bound
        // constraint from self.
        // FIXME: is the fact this could be unbound a footgun?
        let head = Input::new(
            &self.as_dangerous()[..self.len() - tail.len()],
            self.is_bound(),
        );
        // Return the split input parts.
        Ok((head, tail))
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

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<F>(self, mut pred: F) -> (Input<'i>, Input<'i>)
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
                let head = Input::new(head, true);
                // For the tail we derive the bound constaint from self.
                let tail = Input::new(tail, self.is_bound());
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
    ) -> Result<(Input<'i>, Input<'i>), E>
    where
        E: FromContext<'i>,
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
                let head = Input::new(head, true);
                // For the tail we derive the bound constaint from self.
                let tail = Input::new(tail, self.is_bound());
                // Return the split input parts.
                return Ok((head, tail));
            }
        }
        Ok((self.clone(), self.end()))
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: use https://github.com/rust-lang/rust/pull/79135 once stable in 1.51

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
