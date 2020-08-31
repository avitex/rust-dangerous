use core::marker::PhantomData;

use crate::error::{ExpectedLength, ExpectedValue, FromError, Invalid, SealedContext};
use crate::input::Input;

/// A `Reader` is created from and consumes a [`Input`].
pub struct Reader<'i, E = Invalid> {
    input: &'i Input,
    error: PhantomData<E>,
}

impl<'i, E> Reader<'i, E> {
    pub(crate) const fn new(input: &'i Input) -> Self {
        Self {
            input,
            error: PhantomData,
        }
    }

    /// Returns `true` if the reader has no more input to consume.
    #[inline(always)]
    pub fn at_end(&self) -> bool {
        self.input.is_empty()
    }

    /// Skip `len` number of bytes in the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input was not long enough.
    #[inline(always)]
    pub fn skip(&mut self, len: usize) -> Result<(), E>
    where
        E: FromError<ExpectedLength<'i>>,
    {
        self.take(len).map(drop)
    }

    /// Read a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the required length cannot be fullfilled.
    #[inline(always)]
    pub fn take(&mut self, len: usize) -> Result<&'i Input, E>
    where
        E: FromError<ExpectedLength<'i>>,
    {
        let (head, tail) = self.input.split_at(len)?;
        self.input = tail;
        Ok(head)
    }

    /// Read a length of input while a condition remains true,
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn take_while<F>(&mut self, f: F) -> Result<&'i Input, E>
    where
        F: Fn(&'i Input, u8) -> Result<bool, E>,
    {
        let (head, tail) = self.input.split_while(f)?;
        self.input = tail;
        Ok(head)
    }

    /// Peek a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the required length cannot be fullfilled.
    #[inline(always)]
    pub fn peek<F, O>(&self, len: usize, f: F) -> Result<O, E>
    where
        F: FnOnce(&Input) -> Result<O, E>,
        E: FromError<ExpectedLength<'i>>,
        O: 'static,
    {
        let (head, _) = self.input.split_at(len)?;
        f(head)
    }

    /// Returns the next byte in the input without mutating the reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the reader has no more input.
    #[inline(always)]
    pub fn peek_u8(&self) -> Result<u8, E>
    where
        E: FromError<ExpectedLength<'i>>,
    {
        Ok(self.input.first()?)
    }

    /// Returns `true` if `bytes` is next in the input.
    #[inline(always)]
    pub fn peek_eq(&self, bytes: &[u8]) -> bool {
        match self.input.split_at::<Invalid>(bytes.len()) {
            Ok((input, _)) => bytes == input,
            Err(_) => false,
        }
    }

    /// Consume expected bytes from the input.
    ///
    /// Doesn't effect the internal state if the input couldn't
    /// be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes could not be consumed from the input.
    pub fn consume(&mut self, bytes: &'static [u8]) -> Result<(), E>
    where
        E: FromError<ExpectedLength<'i>>,
        E: FromError<ExpectedValue<'i>>,
    {
        match self.input.split_at::<Invalid>(bytes.len()) {
            Ok((input, tail)) if input == bytes => {
                self.input = tail;
                Ok(())
            }
            Ok((input, _)) => Err(E::from_err(ExpectedValue {
                span: input,
                value: crate::input(bytes),
                context: SealedContext {
                    input: self.input,
                    operation: "consume value",
                },
            })),
            Err(_) => Err(E::from_err(ExpectedValue {
                span: self.input.end(),
                value: crate::input(bytes),
                context: SealedContext {
                    input: self.input,
                    operation: "consume value",
                },
            })),
        }
    }

    /// Read a byte, consuming the input.
    ///
    /// # Errors
    ///
    /// Returns an error if there is no more input.
    #[inline(always)]
    pub fn read_u8(&mut self) -> Result<u8, E>
    where
        E: FromError<ExpectedLength<'i>>,
    {
        let (byte, tail) = self.input.split_first()?;
        self.input = tail;
        Ok(byte)
    }

    /// Run a function with the reader with the expectation
    /// all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the function does, or there
    /// is trailing input.
    pub fn read_all<F, O>(&mut self, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Self) -> Result<O, E>,
        E: FromError<E>,
        E: FromError<ExpectedLength<'i>>,
    {
        let complete = self.input;
        let ok = f(self).map_err(|err| {
            E::from_err_ctx(
                err,
                SealedContext {
                    input: complete,
                    operation: "confirm all read",
                },
            )
        })?;
        if self.at_end() {
            Ok(ok)
        } else {
            Err(E::from_err(ExpectedLength {
                min: 0,
                max: Some(0),
                span: self.input,
                context: SealedContext {
                    input: complete,
                    operation: "confirm all read",
                },
            }))
        }
    }

    impl_read_num!(i8, le: read_i8_le, be: read_i8_be);
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

    #[inline(always)]
    pub fn with_error<'r: 'i, T>(&'r mut self) -> Reader<'r, T> {
        Reader {
            input: self.input,
            error: PhantomData,
        }
    }
}
