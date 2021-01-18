use crate::error::{ExpectedLength, ExpectedValid, ExpectedValue, WithContext};
use crate::input::{Bytes, Input, Private, String};

use super::BytesReader;

impl<'i, E> BytesReader<'i, E> {
    /// Skip `len` number of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to skip could not be met.
    #[inline]
    pub fn skip(&mut self, len: usize) -> Result<(), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_at(len, "skip"))
            .map(drop)
    }

    /// Skip a length of input while a predicate check remains true.
    ///
    /// Returns the total length of input skipped.
    pub fn skip_while<F>(&mut self, pred: F) -> usize
    where
        F: FnMut(u8) -> bool,
    {
        self.advance(|input| input.split_while(pred)).len()
    }

    /// Try skip a length of input while a predicate check remains successful
    /// and true.
    ///
    /// Returns the total length of input skipped.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_skip_while<F>(&mut self, pred: F) -> Result<usize, E>
    where
        E: WithContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_while(pred, "try skip while"))
            .map(|head| head.len())
    }

    /// Skip a length of string input while a predicate check remains true.
    ///
    /// Returns the total length of input skipped in bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    pub fn skip_str_while<F>(&mut self, pred: F) -> Result<usize, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> bool,
    {
        self.try_advance(|input| input.split_str_while(pred, "skip str while"))
            .map(|head| head.len())
    }

    /// Try skip a length of string input while a predicate check remains
    /// successful and true.
    ///
    /// Returns the total length of input skipped in bytes.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does, [`ExpectedValid`] if the
    /// the input could never be valid UTF-8 and [`ExpectedLength`] if a UTF-8
    /// code point was cut short.
    pub fn try_skip_str_while<F>(&mut self, pred: F) -> Result<usize, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnMut(char) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_str_while(pred, "try skip str while"))
            .map(|head| head.len())
    }

    /// Read a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to read could not be met.
    pub fn take(&mut self, len: usize) -> Result<Bytes<'i>, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_at(len, "take"))
    }

    /// Read all of the remaining string input.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8
    /// and [`ExpectedLength`] if a UTF-8 code point was cut short.
    #[inline]
    pub fn take_remaining_str(&mut self) -> Result<String<'i>, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_str_while(|_| true, "try take remaining str"))
    }

    /// Read a length of input while a predicate check remains true.
    pub fn take_while<F>(&mut self, pred: F) -> Bytes<'i>
    where
        F: FnMut(u8) -> bool,
    {
        self.advance(|input| input.split_while(pred))
    }

    /// Try read a length of input while a predicate check remains successful
    /// and true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_while<F>(&mut self, pred: F) -> Result<Bytes<'i>, E>
    where
        E: WithContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_while(pred, "try take while"))
    }

    /// Read a length of string input while a predicate check remains true.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short.
    pub fn take_str_while<F>(&mut self, pred: F) -> Result<String<'i>, E>
    where
        E: WithContext<'i>,
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

    /// Peek a length of input.
    ///
    /// The function lifetime is to prevent the peeked `Input` being used as a
    /// value in a parsed structure. Peeked values should only be used in
    /// choosing a correct parse path.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to peek could not be met.
    pub fn peek<'p>(&'p self, len: usize) -> Result<Bytes<'p>, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.input.clone().split_at(len, "peek") {
            Ok((head, _)) => Ok(head),
            Err(err) => Err(err),
        }
    }

    /// Peek a length of input.
    ///
    /// This is equivalent to `peek` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[must_use = "peek result must be used"]
    pub fn peek_opt(&self, len: usize) -> Option<Bytes<'_>> {
        self.input.clone().split_at_opt(len).map(|(head, _)| head)
    }

    /// Returns `true` if `bytes` is next in the `Reader`.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_eq(&self, bytes: &[u8]) -> bool {
        self.input.has_prefix(bytes)
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
        self.input.clone().first("peek u8")
    }

    /// Peek the next byte in the input without mutating the `Reader`.
    ///
    /// This is equivalent to `peek_u8` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_u8_opt(&self) -> Option<u8> {
        self.input.first_opt()
    }

    /// Returns `true` if `byte` is next in the `Reader`.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_u8_eq(&self, byte: u8) -> bool {
        self.peek_eq(&[byte])
    }

    /// Consume expected bytes.
    ///
    /// Doesn't effect the internal state of the `Reader` if the bytes couldn't
    /// be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes could not be consumed.
    pub fn consume(&mut self, bytes: &'i [u8]) -> Result<(), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        self.try_advance(|input| input.split_prefix(bytes, "consume"))
            .map(drop)
    }

    /// Consume optional bytes.
    ///
    /// Returns `true` if the bytes were consumed, `false` if not.
    ///
    /// Doesn't effect the internal state of the `Reader` if the bytes couldn't
    /// be consumed.
    pub fn consume_opt(&mut self, bytes: &[u8]) -> bool {
        self.advance(|input| {
            let (prefix, next) = input.split_prefix_opt(bytes);
            (prefix.is_some(), next)
        })
    }

    /// Consume an expected byte.
    ///
    /// Doesn't effect the internal state of the `Reader` if the byte couldn't
    /// be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the byte could not be consumed.
    pub fn consume_u8(&mut self, byte: u8) -> Result<(), E>
    where
        E: From<ExpectedValue<'i>>,
    {
        self.try_advance(|input| input.split_prefix_u8(byte, "consume u8"))
            .map(drop)
    }

    /// Consume an optional byte.
    ///
    /// Returns `true` if the byte was consumed, `false` if not.
    ///
    /// Doesn't effect the internal state of the `Reader` if the byte couldn't
    /// be consumed.
    pub fn consume_u8_opt(&mut self, byte: u8) -> bool {
        self.consume_opt(&[byte])
    }

    /// Read an array from input.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to read could not be met.
    #[cfg(feature = "const-generics")]
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_array("read array"))
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
            .map(|v| v as i8)
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
