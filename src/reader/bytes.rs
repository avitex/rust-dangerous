use crate::error::{CoreOperation, ExpectedLength, ExpectedValid, WithContext};
use crate::input::{Bytes, PrivateExt, String};

use super::BytesReader;

impl<'i, E> BytesReader<'i, E> {
    /// Read a byte.
    ///
    /// # Errors
    ///
    /// Returns an error if there is no more input.
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_token(CoreOperation::ReadU8))
    }

    /// Read an optional byte.
    ///
    /// Returns `Some(u8)` if there was enough input, `None` if not.
    #[inline]
    pub fn read_u8_opt(&mut self) -> Option<u8> {
        self.advance_opt(PrivateExt::split_token_opt)
    }

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

    /// Read an optional array.
    ///
    /// Returns `Some([u8; N])` if there was enough input, `None` if not.
    #[inline]
    pub fn read_array_opt<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.advance_opt(Bytes::split_array_opt)
    }

    /// Peek the next byte in the input without mutating the `Reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Reader` has no more input.
    #[inline]
    pub fn peek_u8(&self) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.input
            .clone()
            .split_token(CoreOperation::PeekU8)
            .map(|(byte, _)| byte)
    }

    /// Peek the next byte in the input without mutating the `Reader`.
    ///
    /// This is equivalent to `peek_u8` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_u8_opt(&self) -> Option<u8> {
        self.input.clone().split_token_opt().map(|(byte, _)| byte)
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
