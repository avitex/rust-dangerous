use core::slice;
use core::ops::Range;

use crate::error::{Error, Value, ExpectedLength, ExpectedValue};
use crate::input::{input, Input};
use crate::reader::Reader;
use crate::utils::with_context;

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
    pub(crate) fn from_u8(byte: &u8) -> &Input {
        input(slice::from_ref(byte))
    }

    /// Returns an empty `Input` pointing the end of `self`.
    #[inline(always)]
    pub(crate) fn end(&self) -> &Input {
        input(&self.as_dangerous()[self.len()..])
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
        self.as_dangerous().first().copied().ok_or_else(|| {
            E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                input: self,
                operation,
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
        prefix: Value<'i>,
        operation: &'static str,
    ) -> Result<&'i Input, E>
    where
        E: From<ExpectedValue<'i>>,
    {
        let prefix_input = prefix.as_input();
        if self.len() >= prefix_input.len() {
            let bytes = self.as_dangerous();
            let (head, tail) = bytes.split_at(prefix_input.len());
            if head == prefix_input {
                Ok(input(tail))
            } else {
                Err(E::from(ExpectedValue {
                    span: self,
                    value: prefix,
                    input: self,
                    operation,
                }))
            }
        } else {
            Err(E::from(ExpectedValue {
                span: self.end(),
                value: prefix,
                input: self,
                operation,
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
        Ok((head.first(operation)?, tail))
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
        if mid > self.len() {
            Err(E::from(ExpectedLength {
                min: mid,
                max: None,
                span: self,
                input: self,
                operation,
            }))
        } else {
            let (head, tail) = self.as_dangerous().split_at(mid);
            Ok((input(head), input(tail)))
        }
    }

    #[inline(always)]
    pub(crate) fn split_consumed<'i, F, E>(&'i self, mut f: F) -> (&'i Input, &'i Input)
    where
        E: Error<'i>,
        F: FnMut(&mut Reader<'i, E>),
    {
        let mut reader = Reader::new(self);
        f(&mut reader);
        let tail = reader.take_remaining();
        let head = &self.as_dangerous()[..self.len() - tail.len()];
        (input(head), tail)
    }

    #[inline(always)]
    pub(crate) fn try_split_consumed<'i, F, E>(
        &'i self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(&'i Input, &'i Input), E>
    where
        E: Error<'i>,
        F: FnMut(&mut Reader<'i, E>) -> Result<(), E>,
    {
        let mut reader = Reader::new(self);
        with_context(self, operation, || f(&mut reader))?;
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

    /// Trys to split the input while the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn try_split_while<'i, F, E>(
        &'i self,
        mut f: F,
        operation: &'static str,
    ) -> Result<(&'i Input, &'i Input), E>
    where
        E: Error<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let should_continue = with_context(self, operation, || f(*byte))?;
            if !should_continue {
                return Ok((input(head), input(tail)));
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
    #[inline(always)]
    fn as_dangerous_ptr_range(&self) -> Range<*const u8> {
        let bytes = self.as_dangerous();
        let start = bytes.as_ptr();
        // Note: will never wrap, but we are just escaping the use of unsafe
        let end = bytes.as_ptr().wrapping_add(bytes.len());
        start..end
    }

    ///////////////////////////////////////////////////////////////////////////
    // FIXME: replace with const generics when stable

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

    pub(crate) fn split_arr_2<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 2], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(2, operation)?;
        let bytes = head.as_dangerous();
        Ok(([bytes[0], bytes[1]], tail))
    }

    pub(crate) fn split_arr_4<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 4], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(4, operation)?;
        let bytes = head.as_dangerous();
        Ok(([bytes[0], bytes[1], bytes[2], bytes[3]], tail))
    }

    pub(crate) fn split_arr_8<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 8], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(8, operation)?;
        let bytes = head.as_dangerous();
        Ok((
            [
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ],
            tail,
        ))
    }

    pub(crate) fn split_arr_16<'i, E>(
        &'i self,
        operation: &'static str,
    ) -> Result<([u8; 16], &'i Input), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.split_at(16, operation)?;
        let bytes = head.as_dangerous();
        Ok((
            [
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ],
            tail,
        ))
    }
}
