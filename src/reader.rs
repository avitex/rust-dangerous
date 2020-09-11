use core::marker::PhantomData;

use crate::error::{
    Context, Error, ExpectedLength, ExpectedValid, ExpectedValue, Invalid, ToRetryRequirement,
};
use crate::input::Input;

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
        let complete = self.input;
        f(self).map_err(|err| err.with_context(complete, context))
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
        let complete = self.input;
        f(self).map_err(|err| err.with_context(complete, context))
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
        self.context("skip", |r| {
            let (_, tail) = r.input.split_at(len)?;
            r.input = tail;
            Ok(())
        })
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
        F: FnMut(&'i Input, u8) -> bool,
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
        F: FnMut(&'i Input, u8) -> Result<bool, E>,
    {
        self.context("try skip while", |r| {
            let (head, tail) = r.input.try_split_while(pred)?;
            r.input = tail;
            Ok(head.len())
        })
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
        self.context("take", |r| {
            let (head, tail) = r.input.split_at(len)?;
            r.input = tail;
            Ok(head)
        })
    }

    /// Read a length of input while a predicate check remains true.
    pub fn take_while<F>(&mut self, pred: F) -> &'i Input
    where
        F: FnMut(&'i Input, u8) -> bool,
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
        F: FnMut(&'i Input, u8) -> Result<bool, E>,
    {
        self.context("try take while", |r| {
            let (head, tail) = r.input.try_split_while(pred)?;
            r.input = tail;
            Ok(head)
        })
    }

    /// Read all of the input left.
    pub fn take_remaining(&mut self) -> &'i Input {
        let all = self.input;
        self.input = all.end();
        all
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
        O: 'static,
    {
        self.peek_context("peek", |r| {
            let (head, _) = r.input.split_at(len)?;
            Ok(f(head))
        })
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
        self.peek_context("try peek", |r| {
            let (head, _) = r.input.split_at(len)?;
            f(head)
        })
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
        self.peek_context("peek u8", |r| r.input.first())
    }

    /// Returns `true` if `bytes` is next in the input.
    #[inline]
    pub fn peek_eq(&self, bytes: &[u8]) -> bool {
        self.input.split_prefix::<Invalid>(bytes).is_ok()
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
        self.context("consume", |r| {
            let tail = r.input.split_prefix::<E>(bytes)?;
            r.input = tail;
            Ok(())
        })
    }

    /// Read a value with any error's details erased except for an optional
    /// [`RetryRequirement`](crate::RetryRequirement).
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
    ///         r.read_erased("ipv4 addr", |i| {
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
    pub fn read_erased<F, O, R>(&mut self, expected: &'static str, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Self) -> Result<O, R>,
        E: Error<'i>,
        E: From<ExpectedValid<'i>>,
        R: ToRetryRequirement,
    {
        f(self).map_err(|err| {
            E::from(ExpectedValid {
                expected,
                span: self.input,
                input: self.input,
                operation: "read erased",
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
        self.context("read u8", |r| {
            let (byte, tail) = r.input.split_first::<E>()?;
            r.input = tail;
            Ok(byte)
        })
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
