use crate::error::{ExpectedLength, ExpectedValid, WithContext};
use crate::input::{PrivateExt, String};

use super::BytesReader;

impl<'i, E> BytesReader<'i, E> {
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
        self.try_advance(|input| input.split_str_while(pred, "skip str while"))
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
        self.try_advance(|input| input.try_split_str_while(pred, "try skip str while"))
            .map(drop)
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
        self.try_advance(|input| input.split_str_while(|_| true, "take remaining str"))
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
        self.try_advance(|input| input.split_str_while(pred, "take str while"))
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
        self.try_advance(|input| input.try_split_str_while(pred, "try take str while"))
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
            .split_first("peek u8")
            .map(|(byte, _)| byte)
    }

    /// Peek the next byte in the input without mutating the `Reader`.
    ///
    /// This is equivalent to `peek_u8` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_u8_opt(&self) -> Option<u8> {
        self.input.clone().split_first_opt().map(|(byte, _)| byte)
    }

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
        self.try_advance(|input| input.split_first("read u8"))
    }

    /// Read a `i8`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is no more input.
    #[inline]
    pub fn read_i8(&mut self) -> Result<i8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_first("read i8"))
            .map(|v| i8::from_ne_bytes([v]))
    }

    impl_read_num!(u16, le: read_u16_le, be: read_u16_be);
    impl_read_num!(i16, le: read_i16_le, be: read_i16_be);
    impl_read_num!(u32, le: read_u32_le, be: read_u32_be);
    impl_read_num!(i32, le: read_i32_le, be: read_i32_be);
    impl_read_num!(u64, le: read_u64_le, be: read_u64_be);
    impl_read_num!(i64, le: read_i64_le, be: read_i64_be);
    impl_read_num!(u128, le: read_u128_le, be: read_u128_be);
    impl_read_num!(i128, le: read_i128_le, be: read_i128_be);
    impl_read_num!(f32, le: read_f32_le, be: read_f32_be);
    impl_read_num!(f64, le: read_f64_le, be: read_f64_be);
}
