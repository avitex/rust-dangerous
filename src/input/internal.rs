use core::convert::TryInto;
use core::slice;

use crate::error::{
    with_context, ExpectedContext, ExpectedLength, ExpectedValid, ExpectedValue, FromContext,
    OperationContext, ToRetryRequirement, Value,
};
use crate::reader::Reader;

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

impl Input {
    #[inline(always)]
    pub(crate) fn from_u8(byte: &u8) -> &Input {
        input(slice::from_ref(byte))
    }

    /// Returns an empty `Input` pointing the end of `self`.
    #[inline(always)]
    pub(crate) fn end(&self) -> &Input {
        input(&self.as_dangerous()[self.len()..])
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
    pub(crate) fn first<'i, E>(&'i self, operation: &'static str) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.first_opt().ok_or_else(|| {
            E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "a byte",
                },
            })
        })
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

    #[inline(always)]
    pub(crate) fn split_prefix<'i, E>(
        &'i self,
        prefix_value: Value<'i>,
        operation: &'static str,
    ) -> Result<&'i Input, E>
    where
        E: From<ExpectedValue<'i>>,
    {
        let prefix = prefix_value.as_input();
        let (maybe_prefix, tail) = self.split_max(prefix.len());
        if maybe_prefix == prefix {
            Ok(tail)
        } else {
            Err(E::from(ExpectedValue {
                actual: maybe_prefix,
                expected: prefix_value,
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "exact value",
                },
            }))
        }
    }

    /// Splits the input into the first byte and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn split_first<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<(u8, &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(1, operation)?;
        Ok((head.as_dangerous()[0], tail))
    }

    /// Splits the input into two at `mid`.
    #[inline(always)]
    pub(crate) fn split_at_opt(&self, mid: usize) -> Option<(&Input, &Input)> {
        if mid > self.len() {
            None
        } else {
            let (head, tail) = self.as_dangerous().split_at(mid);
            Some((input(head), input(tail)))
        }
    }

    /// Splits the input into two at `mid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `mid > self.len()`.
    #[inline(always)]
    pub(crate) fn split_at<'i, E>(
        &'i self,
        mid: usize,
        operation: &'static str,
    ) -> Result<(&'i Input, &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.split_at_opt(mid).ok_or_else(|| {
            E::from(ExpectedLength {
                min: mid,
                max: None,
                span: self,
                input: self,
                context: ExpectedContext {
                    operation,
                    expected: "enough input",
                },
            })
        })
    }

    #[inline(always)]
    pub(crate) fn split_consumed<'i, F, E>(&'i self, mut f: F) -> (&'i Input, &'i Input)
    where
        E: FromContext<'i>,
        F: FnMut(&mut Reader<'i, E>),
    {
        let mut reader = Reader::new(self);
        f(&mut reader);
        let tail = reader.take_remaining();
        let head = &self.as_dangerous()[..self.len() - tail.len()];
        (input(head), tail)
    }

    #[inline(always)]
    pub(crate) fn split_expect<'i, F, T, E>(
        &'i self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, &'i Input), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Option<T>,
    {
        let mut reader = Reader::new(self);
        match f(&mut reader) {
            Some(ok) => Ok((ok, reader.take_remaining())),
            None => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span: input(span),
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
    pub(crate) fn try_split_expect<'i, F, T, E>(
        &'i self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, &'i Input), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Result<Option<T>, E>,
    {
        let context = ExpectedContext {
            expected,
            operation,
        };
        let mut reader = Reader::new(self);
        match with_context(self, context, || f(&mut reader))? {
            Some(ok) => Ok((ok, reader.take_remaining())),
            None => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span: input(span),
                    input: self,
                    context,
                    retry_requirement: None,
                }))
            }
        }
    }

    #[inline(always)]
    pub(crate) fn try_split_expect_erased<'i, F, T, R, E>(
        &'i self,
        f: F,
        expected: &'static str,
        operation: &'static str,
    ) -> Result<(T, &'i Input), E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Reader<'i, E>) -> Result<T, R>,
        R: ToRetryRequirement,
    {
        let mut reader = Reader::new(self);
        match f(&mut reader) {
            Ok(ok) => Ok((ok, reader.take_remaining())),
            Err(err) => {
                let tail = reader.take_remaining();
                let span = &self.as_dangerous()[..self.len() - tail.len()];
                Err(E::from(ExpectedValid {
                    span: input(span),
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
    pub(crate) fn try_split_consumed<'i, F, E>(
        &'i self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(&'i Input, &'i Input), E>
    where
        E: FromContext<'i>,
        F: FnMut(&mut Reader<'i, E>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self);
        with_context(self, OperationContext(operation), || f(&mut reader))?;
        let tail = reader.take_remaining();
        let head = &self.as_dangerous()[..self.len() - tail.len()];
        Ok((input(head), tail))
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

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<F>(&self, mut f: F) -> (&Input, &Input)
    where
        F: FnMut(u8) -> bool,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let should_continue = f(*byte);
            if !should_continue {
                return (input(head), input(tail));
            }
        }
        (self, self.end())
    }

    /// Tries to split the input while the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn try_split_while<'i, F, E>(
        &'i self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(&'i Input, &'i Input), E>
    where
        E: FromContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let should_continue = with_context(self, OperationContext(operation), || f(*byte))?;
            if !should_continue {
                return Ok((input(head), input(tail)));
            }
        }
        Ok((self, self.end()))
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: replace with const generics when stable

    #[inline(always)]
    pub(crate) fn split_arr_1<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 1], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (byte, tail) = self.split_first(operation)?;
        Ok(([byte], tail))
    }

    #[inline(always)]
    pub(crate) fn split_arr_2<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 2], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(2, operation)?;
        Ok((head.as_dangerous().try_into().unwrap(), tail))
    }

    #[inline(always)]
    pub(crate) fn split_arr_4<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 4], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(4, operation)?;
        Ok((head.as_dangerous().try_into().unwrap(), tail))
    }

    #[inline(always)]
    pub(crate) fn split_arr_8<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 8], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(8, operation)?;
        Ok((head.as_dangerous().try_into().unwrap(), tail))
    }

    #[inline(always)]
    pub(crate) fn split_arr_16<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 16], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(16, operation)?;
        Ok((head.as_dangerous().try_into().unwrap(), tail))
    }
}
