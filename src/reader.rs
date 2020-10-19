use core::fmt;
use core::marker::PhantomData;

use crate::error::{
    with_context, Context, ExpectedLength, ExpectedValid, ExpectedValue, FromContext,
    OperationContext, ToRetryRequirement, Value,
};
use crate::input::Input;

/// A `Reader` is created from and consumes an [`Input`].
///
/// You can only create a [`Reader`] from [`Input`] via [`Input::read_all()`],
/// [`Input::read_partial()`] or [`Input::read_infallible()`].
///
/// # Errors
///
/// Functions on `Reader` are designed to provide a panic free interface and if
/// applicable, clearly define the type of error that can can be thown.
///
/// If want to verify input and optionally return a type from that verification,
/// [`verify()`], [`try_verify()`], [`expect()`], [`try_expect()`] and
/// [`try_expect_erased()`] is provided. These functions are the interface for
/// creating errors based of what was expected.
///
/// [`try_expect_erased()`] is provided for reading a custom type that does not
/// support a `&mut Reader<'i, E>` interface, for example a type implementing
/// `FromStr`.
///
/// [`recover()`] and [`recover_if()`] are provided as an escape hatch when you
/// wish to catch an error and try another branch.
///
/// [`context()`] and [`peek_context()`] are provided to add a [`Context`] to
/// any error thrown inside their scope. This is useful for debugging.
///
/// # Peeking
///
/// Peeking should be used to find the correct path to consume. Values read from
/// peeking should not be used for the resulting type.
///
/// ```rust
/// use dangerous::Invalid;
///
/// let input = dangerous::input(b"true");
/// let result: Result<_, Invalid> = input.read_all(|r| {
///     // We use `peek_u8` here because we expect at least one byte.
///     // If we wanted to handle the case when there is no more input left,
///     // for example to provide a default, we would use `peek_u8_opt`.
///     // This means we can handle a `RetryRequirement` if the `Reader` is
///     // at the end of the input.s
///     r.try_expect("boolean", |r| match r.peek_u8()? {
///         b't' => r.consume(b"true").map(|()| Some(true)),
///         b'f' => r.consume(b"false").map(|()| Some(false)),
///         _ => Ok(None),
///     })
/// });
/// assert!(matches!(result, Ok(true)));
/// ```
///
/// [`context()`]: Reader::context()  
/// [`peek_context()`]: Reader::peek_context()  
/// [`verify()`]: Reader::verify()  
/// [`try_verify()`]: Reader::try_verify()  
/// [`expect()`]: Reader::expect()  
/// [`try_expect()`]: Reader::try_expect()  
/// [`try_expect_erased()`]: Reader::try_expect_erased()  
/// [`recover()`]: Reader::recover()  
/// [`recover_if()`]: Reader::recover_if()  
pub struct Reader<'i, E> {
    input: &'i Input,
    error: PhantomData<E>,
}

impl<'i, E> Reader<'i, E> {
    /// Mutably use the `Reader` with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function with the specified
    /// context attached.
    pub fn context<C, F, T>(&mut self, context: C, f: F) -> Result<T, E>
    where
        E: FromContext<'i>,
        C: Context,
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        with_context(self.input, context, || f(self))
    }

    /// Immutably use the `Reader` with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function with the specified
    /// context attached.
    pub fn peek_context<C, F, T>(&self, context: C, f: F) -> Result<T, E>
    where
        E: FromContext<'i>,
        C: Context,
        F: FnOnce(&Self) -> Result<T, E>,
    {
        with_context(self.input, context, || f(self))
    }

    /// Returns `true` if the `Reader` has no more input to consume.
    #[inline]
    pub fn at_end(&self) -> bool {
        self.input.is_empty()
    }

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
        let (_, tail) = self.input.split_at(len, "skip")?;
        self.input = tail;
        Ok(())
    }

    /// Skip a length of input while a predicate check remains true.
    ///
    /// Returns the total length of input skipped.
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
    /// Returns the total length of input skipped.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_skip_while<F>(&mut self, pred: F) -> Result<usize, E>
    where
        E: FromContext<'i>,
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
    /// Returns an error if the length requirement to read could not be met.
    pub fn take(&mut self, len: usize) -> Result<&'i Input, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, tail) = self.input.split_at(len, "take")?;
        self.input = tail;
        Ok(head)
    }

    /// Read all of the remaining input.
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
        E: FromContext<'i>,
        F: FnMut(u8) -> Result<bool, E>,
    {
        let (head, tail) = self.input.try_split_while(pred, "try take while")?;
        self.input = tail;
        Ok(head)
    }

    /// Read a length of input that was successfully consumed from a sub-parse.
    pub fn take_consumed<F>(&mut self, consumer: F) -> &'i Input
    where
        E: FromContext<'i>,
        F: FnMut(&mut Self),
    {
        let (head, tail) = self.input.split_consumed(consumer);
        self.input = tail;
        head
    }

    /// Try read a length of input that was successfully consumed from a
    /// sub-parse.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Invalid;
    ///
    /// let consumed = dangerous::input(b"abc").read_all::<_, _, Invalid>(|r| {
    ///     r.try_take_consumed(|r| {
    ///         r.skip(1)?;
    ///         r.consume(b"bc")
    ///     })
    /// }).unwrap();
    ///
    /// assert_eq!(consumed, &b"abc"[..]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_consumed<F>(&mut self, consumer: F) -> Result<&'i Input, E>
    where
        E: FromContext<'i>,
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
    /// The function lifetime is to prevent the peeked `Input` being used as a
    /// value in a parsed structure. Peeked values should only be used in
    /// choosing a correct parse path.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to peek could not be met.
    pub fn peek<'p>(&'p self, len: usize) -> Result<&'p Input, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        let (head, _) = self.input.split_at(len, "peek")?;
        Ok(head)
    }

    /// Peek a length of input.
    ///
    /// This is equivalent to `peek` but does not return an error.
    pub fn peek_opt(&self, len: usize) -> Option<&Input> {
        self.input.split_at_opt(len).map(|(head, _)| head)
    }

    /// Returns `true` if `bytes` is next in the `Reader`.
    #[inline]
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
        self.input.first("peek u8")
    }

    /// Peek the next byte in the input without mutating the `Reader`.
    ///
    /// This is equivalent to `peek_u8` but does not return an error.
    #[inline]
    pub fn peek_u8_opt(&self) -> Option<u8> {
        self.input.first_opt()
    }

    /// Returns `true` if `byte` is next in the `Reader`.
    #[inline]
    pub fn peek_u8_eq(&self, byte: u8) -> bool {
        self.input.has_prefix(&[byte])
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
        let prefix = Value::Bytes(bytes);
        let tail = self.input.split_prefix::<E>(prefix, "consume")?;
        self.input = tail;
        Ok(())
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
        let prefix = Value::Byte(byte);
        let tail = self.input.split_prefix::<E>(prefix, "consume u8")?;
        self.input = tail;
        Ok(())
    }

    /// Read and verify a value without returning it.
    ///
    /// # Errors
    ///
    /// Returns an error if the verifier function returned `false`.
    pub fn verify<F>(&mut self, expected: &'static str, verifier: F) -> Result<(), E>
    where
        F: FnOnce(&mut Self) -> bool,
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        let ((), tail) = self.input.split_expect(
            |r: &mut Self| {
                if verifier(r) {
                    Some(())
                } else {
                    None
                }
            },
            expected,
            "verify",
        )?;
        self.input = tail;
        Ok(())
    }

    /// Try read and verify a value without returning it.
    ///
    /// # Errors
    ///
    /// Returns an error if the verifier function returned `false` or an error.
    pub fn try_verify<F>(&mut self, expected: &'static str, verifier: F) -> Result<(), E>
    where
        F: FnOnce(&mut Self) -> Result<bool, E>,
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        let ((), tail) = self.input.try_split_expect(
            |r: &mut Self| {
                if verifier(r)? {
                    Ok(Some(()))
                } else {
                    Ok(None)
                }
            },
            expected,
            "try verify",
        )?;
        self.input = tail;
        Ok(())
    }

    /// Expect a value to be read and returned as `Some(T)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the returned value was `None`.
    pub fn expect<F, T>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Option<T>,
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        let (ok, tail) = self.input.split_expect(f, expected, "expect")?;
        self.input = tail;
        Ok(ok)
    }

    /// Expect a value to be read successfully and returned as `Some(O)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the returned value was `None` or if the provided
    /// function does.
    pub fn try_expect<F, T>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Self) -> Result<Option<T>, E>,
    {
        let (ok, tail) = self.input.try_split_expect(f, expected, "try expect")?;
        self.input = tail;
        Ok(ok)
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
    /// use dangerous::{Error, Expected, Invalid, Reader};
    ///
    /// // Our custom reader function
    /// fn read_ipv4_addr<'i, E>(r: &mut Reader<'i, E>) -> Result<Ipv4Addr, E>
    /// where
    ///   E: Error<'i>,
    /// {
    ///     r.try_expect_erased("ipv4 addr", |i| {
    ///         i.take_remaining()
    ///             .to_dangerous_str()
    ///             .and_then(|s| s.parse().map_err(|_| Invalid::fatal()))
    ///     })
    /// }
    ///
    /// let input = dangerous::input(b"192.168.1.x");
    /// let error: Expected = input.read_all(read_ipv4_addr).unwrap_err();
    ///
    /// // Prefer string input formatting
    /// println!("{:#}", error);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if provided function does.
    ///
    /// [`RetryRequirement`]: crate::error::RetryRequirement
    pub fn try_expect_erased<F, T, R>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        E: FromContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Self) -> Result<T, R>,
        R: ToRetryRequirement,
    {
        let (ok, tail) = self
            .input
            .try_split_expect_erased(f, expected, "try expect erased")?;
        self.input = tail;
        Ok(ok)
    }

    /// Recovers from an error returning `Some(O)` if successful, or `None` if
    /// an error occurred.
    ///
    /// If an error is recovered from the `Reader`'s internal state is reset.
    #[inline]
    pub fn recover<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let complete = self.input;
        match f(self) {
            Ok(ok) => Some(ok),
            Err(_) => {
                self.input = complete;
                None
            }
        }
    }

    /// Recovers from an error based on a predicate.
    ///
    /// If an error is recovered from the `Reader`'s internal state is reset.
    ///
    /// If an error occurs and the predicate returns `true` the error is
    /// recovered, `Ok(None)` is returned.
    ///
    /// # Errors
    ///
    /// If an error occurs and the predicate returns `false` the error is not
    /// recovered, `Err(E)` is returned.
    #[inline]
    pub fn recover_if<F, T, R>(&mut self, f: F, pred: R) -> Result<Option<T>, E>
    where
        E: FromContext<'i>,
        F: FnOnce(&mut Self) -> Result<T, E>,
        R: FnOnce(&E) -> bool,
    {
        let complete = self.input;
        match f(self) {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => {
                if pred(&err) {
                    self.input = complete;
                    Ok(None)
                } else {
                    Err(err.from_context(complete, OperationContext("try recover")))
                }
            }
        }
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

impl<'i, E> fmt::Debug for Reader<'i, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("input", &self.input)
            .finish()
    }
}
