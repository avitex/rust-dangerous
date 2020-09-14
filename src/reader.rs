use core::marker::PhantomData;

use crate::error::{
    Context, Error, ExpectedContext, ExpectedLength, ExpectedValid, ExpectedValue,
    ToRetryRequirement, Value,
};
use crate::input::Input;
use crate::utils::{with_context, with_operation_context};

/// A `Reader` is created from and consumes a [`Input`].
pub struct Reader<'i, E> {
    input: &'i Input,
    error: PhantomData<E>,
}

impl<'i, E> Reader<'i, E>
where
    E: Error<'i>,
{
    /// Mutably use the reader with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function, and attaches the
    /// specified context to it.
    pub fn context<C, F, O>(&mut self, context: C, f: F) -> Result<O, E>
    where
        C: Context,
        F: FnOnce(&mut Self) -> Result<O, E>,
    {
        with_context(self.input, context, || f(self))
    }

    /// Immutably use the reader with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function, and attaches the
    /// specified context to it.
    pub fn peek_context<C, F, O>(&self, context: C, f: F) -> Result<O, E>
    where
        C: Context,
        F: FnOnce(&Self) -> Result<O, E>,
    {
        with_context(self.input, context, || f(self))
    }

    /// Returns `true` if the reader has no more input to consume.
    #[inline]
    pub fn at_end(&self) -> bool {
        self.input.is_empty()
    }

    /// Skip `len` number of bytes in the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input was not long enough.
    #[inline]
    pub fn skip(&mut self, len: usize) -> Result<(), E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (_, tail) = self.input.split_at(len, "skip")?;
        self.input = tail;
        Ok(())
    }

    /// Skip a length of input while a predicate check remains true.
    ///
    /// Returns the length of input skipped.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn skip_while<F>(&mut self, pred: F) -> usize
    where
        F: FnMut(u8) -> bool,
    {
        let (head, tail) = self.input.split_while(pred);
        self.input = tail;
        head.len()
    }

    /// Try skip a length of input while a predicate check remains successful
    /// and true.
    ///
    /// Returns the length of input skipped.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_skip_while<F>(&mut self, pred: F) -> Result<usize, E>
    where
        F: FnMut(u8) -> Result<bool, E>,
    {
        let (head, tail) = self.input.try_split_while(pred, "try skip while")?;
        self.input = tail;
        Ok(head.len())
    }

    /// Read a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the required length cannot be fullfilled.
    pub fn take(&mut self, len: usize) -> Result<&'i Input, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.input.split_at(len, "take")?;
        self.input = tail;
        Ok(head)
    }

    /// Read all of the input left.
    pub fn take_remaining(&mut self) -> &'i Input {
        let all = self.input;
        self.input = all.end();
        all
    }

    /// Read a length of input while a predicate check remains true.
    pub fn take_while<F>(&mut self, pred: F) -> &'i Input
    where
        F: FnMut(u8) -> bool,
    {
        let (head, tail) = self.input.split_while(pred);
        self.input = tail;
        head
    }

    /// Try read a length of input while a predicate check remains successful
    /// and true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_while<F>(&mut self, pred: F) -> Result<&'i Input, E>
    where
        F: FnMut(u8) -> Result<bool, E>,
    {
        let (head, tail) = self.input.try_split_while(pred, "try take while")?;
        self.input = tail;
        Ok(head)
    }

    /// Read a length of input that was successfully parsed.
    pub fn take_consumed<F>(&mut self, consumer: F) -> &'i Input
    where
        F: FnMut(&mut Self),
    {
        let (head, tail) = self.input.split_consumed(consumer);
        self.input = tail;
        head
    }

    /// Try read a length of input that was successfully parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided function does.
    pub fn try_take_consumed<F>(&mut self, consumer: F) -> Result<&'i Input, E>
    where
        F: FnMut(&mut Self) -> Result<(), E>,
    {
        let (head, tail) = self
            .input
            .try_split_consumed(consumer, "try take consumed")?;
        self.input = tail;
        Ok(head)
    }

    /// Peek a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the required length cannot be fullfilled.
    pub fn peek<F, O>(&self, len: usize, f: F) -> Result<O, E>
    where
        F: FnOnce(&Input) -> O,
        E: From<ExpectedLength<'i>>,
    {
        let (head, _) = self.input.split_at(len, "peek")?;
        Ok(f(head))
    }

    /// Try peek a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the required length cannot be fullfilled,
    /// or if the provided function returns one.
    pub fn try_peek<F, O>(&self, len: usize, f: F) -> Result<O, E>
    where
        F: FnOnce(&'i Input) -> Result<O, E>,
        E: From<ExpectedLength<'i>>,
        O: 'static,
    {
        let (head, _) = self.input.split_at(len, "try peek")?;
        with_operation_context(self.input, "try peek", || f(head))
    }

    /// Returns the next byte in the input without mutating the reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the reader has no more input.
    #[inline]
    pub fn peek_u8(&self) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.input.first("peek u8")
    }

    /// Returns `true` if `bytes` is next in the input.
    #[inline]
    pub fn peek_eq(&self, bytes: &[u8]) -> bool {
        self.input.has_prefix(bytes)
    }

    /// Consume expected bytes from the input.
    ///
    /// Doesn't effect the internal state if the input couldn't be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes could not be consumed from the input.
    pub fn consume(&mut self, bytes: &'i [u8]) -> Result<(), E>
    where
        E: From<ExpectedLength<'i>>,
        E: From<ExpectedValue<'i>>,
    {
        let prefix = Value::Bytes(bytes);
        let tail = self.input.split_prefix::<E>(prefix, "consume")?;
        self.input = tail;
        Ok(())
    }

    /// Consume expected bytes from the input.
    ///
    /// Doesn't effect the internal state if the input couldn't be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes could not be consumed from the input.
    pub fn consume_u8(&mut self, byte: u8) -> Result<(), E>
    where
        E: From<ExpectedLength<'i>>,
        E: From<ExpectedValue<'i>>,
    {
        let prefix = Value::Byte(byte);
        let tail = self.input.split_prefix::<E>(prefix, "consume u8")?;
        self.input = tail;
        Ok(())
    }

    /// Expect a value to be read and returned as `Some`.
    ///
    /// # Errors
    ///
    /// Returns an error if the returned value was `None`.
    pub fn expect<F, O>(&mut self, expected: &'static str, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Self) -> Option<O>,
        E: Error<'i>,
        E: From<ExpectedValid<'i>>,
    {
        match f(self) {
            Some(ok) => Ok(ok),
            None => Err(E::from(ExpectedValid {
                span: self.input,
                input: self.input,
                context: ExpectedContext {
                    expected,
                    operation: "expect",
                },
                retry_requirement: None,
            })),
        }
    }

    /// Expect a value to be read successfully and returned as `Some`.
    ///
    /// # Errors
    ///
    /// Returns an error if failed to read, or the returned value was `None`.
    pub fn try_expect<F, O>(&mut self, expected: &'static str, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Self) -> Result<Option<O>, E>,
        E: Error<'i>,
        E: From<ExpectedValid<'i>>,
    {
        let context = ExpectedContext {
            expected,
            operation: "try expect",
        };
        match with_context(self.input, context, || f(self))? {
            Some(ok) => Ok(ok),
            None => Err(E::from(ExpectedValid {
                span: self.input,
                input: self.input,
                context,
                retry_requirement: None,
            })),
        }
    }

    /// Expect a value with any error's details erased except for an optional
    /// [`RetryRequirement`].
    ///
    /// This function is useful for reading custom/unsupported types easily
    /// without having to create custom errors.
    ///
    /// # Example
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// use dangerous::{Error, Invalid, Expected, ExpectedLength, ExpectedValid};
    ///
    /// // Our custom reader function
    /// fn read_ipv4_addr<'i, E>(input: &'i dangerous::Input) -> Result<Ipv4Addr, E>
    /// where
    ///   E: Error<'i>,
    ///   E: From<ExpectedValid<'i>>,
    ///   E: From<ExpectedLength<'i>>,
    /// {
    ///     input.read_all(|r| {
    ///         r.try_expect_erased("ipv4 addr", |i| {
    ///             i.take_remaining()
    ///                 .to_dangerous_str()
    ///                 .and_then(|s| s.parse().map_err(|_| Invalid::fatal()))
    ///         })
    ///     })
    /// }
    ///
    /// let input = dangerous::input(b"192.168.1.x");
    /// let error: Expected = read_ipv4_addr(input).unwrap_err();
    /// println!("{}", error);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    ///
    /// [`RetryRequirement`]: crate::RetryRequirement
    pub fn try_expect_erased<F, O, R>(&mut self, expected: &'static str, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Self) -> Result<O, R>,
        E: Error<'i>,
        E: From<ExpectedValid<'i>>,
        R: ToRetryRequirement,
    {
        f(self).map_err(|err| {
            E::from(ExpectedValid {
                span: self.input,
                input: self.input,
                context: ExpectedContext {
                    expected,
                    operation: "try expect erased",
                },
                retry_requirement: err.to_retry_requirement(),
            })
        })
    }

    /// Read a byte, consuming the input.
    ///
    /// # Errors
    ///
    /// Returns an error if there is no more input.
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (byte, tail) = self.input.split_first::<E>("read u8")?;
        self.input = tail;
        Ok(byte)
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

    /// Create a sub reader with a given error type.
    #[inline]
    pub fn error<T>(&mut self) -> Reader<'_, T> {
        Reader {
            input: self.input,
            error: PhantomData,
        }
    }

    /// Create a `Reader` given `Input`.
    pub(crate) fn new(input: &'i Input) -> Self {
        Self {
            input,
            error: PhantomData,
        }
    }
}
