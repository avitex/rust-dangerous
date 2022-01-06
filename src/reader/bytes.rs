use crate::error::{CoreOperation, ExpectedLength, ExpectedValid, WithContext};
use crate::input::{Bytes, String};

use super::BytesReader;

impl<'i, E> BytesReader<'i, E> {
    /// Read an array from input.
    ///
    /// # Integers
    ///
    /// This function can be used to read integers like so:
    ///
    /// ```
    /// use dangerous::{Input, Invalid};
    ///
    /// let result: Result<_, Invalid> = dangerous::input(&[1, 0, 0, 0]).read_all(|r| {
    ///     r.read_array().map(u32::from_le_bytes)
    /// });
    ///
    /// assert_eq!(result.unwrap(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to read could not be met.
    #[inline]
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_array(CoreOperation::ReadArray))
    }

    /// Read a reference to an array from input.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to read could not be met.
    #[inline]
    pub fn read_array_ref<const N: usize>(&mut self) -> Result<&'i [u8; N], E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_array_ref(CoreOperation::ReadArray))
    }

    /// Read an optional array.
    ///
    /// Returns `Some([u8; N])` if there was enough input, `None` if not.
    #[inline]
    pub fn read_array_opt<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.advance_opt(Bytes::split_array_opt)
    }

    /// Read an optional reference to an array.
    ///
    /// Returns `Some(&[u8; N])` if there was enough input, `None` if not.
    #[inline]
    pub fn read_array_ref_opt<const N: usize>(&mut self) -> Option<&'i [u8; N]> {
        self.advance_opt(Bytes::split_array_ref_opt)
    }

    /// Read the remaining string input.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    pub fn take_remaining_str(&mut self) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_str_while(|_| true, CoreOperation::TakeRemainingStr))
    }

    /// Read a length of string input while a predicate check remains true.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    pub fn take_str_while<F>(&mut self, pred: F) -> Result<String<'i>, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> bool,
    {
        self.try_advance(|input| input.split_str_while(pred, CoreOperation::TakeStrWhile))
    }

    /// Try read a length of string input while a predicate check remains true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does, [`ExpectedValid`] if the
    /// the input could never be valid UTF-8 and [`ExpectedLength`] if a UTF-8
    /// code point was cut short.
    pub fn try_take_str_while<F>(&mut self, pred: F) -> Result<String<'i>, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_str_while(pred, CoreOperation::TakeStrWhile))
    }

    /// Skip a length of string input while a predicate check remains true.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    pub fn skip_str_while<F>(&mut self, pred: F) -> Result<(), E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> bool,
    {
        self.try_advance(|input| input.split_str_while(pred, CoreOperation::SkipStrWhile))
            .map(drop)
    }

    /// Try skip a length of string input while a predicate check remains
    /// successful and true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does, [`ExpectedValid`] if the
    /// the input could never be valid UTF-8 and [`ExpectedLength`] if a UTF-8
    /// code point was cut short.
    pub fn try_skip_str_while<F>(&mut self, pred: F) -> Result<(), E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_str_while(pred, CoreOperation::SkipStrWhile))
            .map(drop)
    }
}
