mod bytes;

use core::marker::PhantomData;

#[cfg(feature = "retry")]
use crate::error::ToRetryRequirement;
use crate::error::{with_context, Context, ExpectedValid, OperationContext, WithContext};
use crate::fmt;
use crate::input::{Bytes, Input, PrivateExt, String};

/// [`Bytes`] specific [`Reader`].
pub type BytesReader<'i, E> = Reader<'i, E, Bytes<'i>>;

/// [`String`] specific [`Reader`].
pub type StringReader<'i, E> = Reader<'i, E, String<'i>>;

/// A `Reader` is created from and consumes an [`Input`].
///
/// You can only create a [`Reader`] from [`Input`] via [`Input::read_all()`],
/// [`Input::read_partial()`] or [`Input::read_infallible()`].
///
/// See [`BytesReader`] for [`Bytes`] specific functions and [`StringReader`]
/// for [`String`] specific functions.
///
/// # Errors
///
/// Functions on `Reader` are designed to provide a panic free interface and if
/// applicable, clearly define the type of error that can can be thown.
///
/// To verify input and optionally return a type from that verification,
/// [`verify()`], [`try_verify()`], [`expect()`], [`try_expect()`] and
/// [`try_expect_erased()`] is provided. These functions are the interface for
/// creating errors based off what was expected.
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
/// ```
/// use dangerous::{Input, Invalid};
///
/// let input = dangerous::input(b"true");
/// let result: Result<_, Invalid> = input.read_all(|r| {
///     // We use `peek_u8` here because we expect at least one byte.
///     // If we wanted to handle the case when there is no more input left,
///     // for example to provide a default, we would use `peek_u8_opt`.
///     // The below allows us to handle a `RetryRequirement` if the
///     // `Reader` is at the end of the input.
///     r.try_expect("boolean", |r| match r.peek_u8()? {
///         b't' => r.consume(b"true").map(|()| Some(true)),
///         b'f' => r.consume(b"false").map(|()| Some(false)),
///         _ => Ok(None),
///     })
/// });
/// assert!(matches!(result, Ok(true)));
/// ```
///
/// [`Input`]: crate::input::Input  
/// [`Input::read_all()`]: crate::input::Input::read_all()  
/// [`Input::read_partial()`]: crate::input::Input::read_partial()  
/// [`Input::read_infallible()`]: crate::input::Input::read_infallible()  
/// [`context()`]: Reader::context()  
/// [`peek_context()`]: Reader::peek_context()  
/// [`verify()`]: Reader::verify()  
/// [`try_verify()`]: Reader::try_verify()  
/// [`expect()`]: Reader::expect()  
/// [`try_expect()`]: Reader::try_expect()  
/// [`try_expect_erased()`]: Reader::try_expect_erased()  
/// [`recover()`]: Reader::recover()  
/// [`recover_if()`]: Reader::recover_if()  
/// [`RetryRequirement`]: crate::error::RetryRequirement  
pub struct Reader<'i, E, I>
where
    I: Input<'i>,
{
    input: I,
    types: PhantomData<(&'i (), E)>,
}

impl<'i, E, I> Reader<'i, E, I>
where
    I: Input<'i>,
{
    /// Returns `true` if the `Reader` has no more input to consume.
    #[must_use]
    #[inline(always)]
    pub fn at_end(&self) -> bool {
        self.input.is_empty()
    }

    /// Read all of the remaining input.
    #[inline(always)]
    pub fn take_remaining(&mut self) -> I {
        self.advance(|input| (input.clone(), input.end()))
    }

    /// Mutably use the `Reader` with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function with the specified
    /// context attached.
    #[inline(always)]
    pub fn context<F, T>(&mut self, context: impl Context, f: F) -> Result<T, E>
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        with_context(self.input.clone(), context, || f(self))
    }

    /// Immutably use the `Reader` with a given context.
    ///
    /// # Errors
    ///
    /// Returns any error returned by the provided function with the specified
    /// context attached.
    #[inline(always)]
    pub fn peek_context<F, T>(&self, context: impl Context, f: F) -> Result<T, E>
    where
        E: WithContext<'i>,
        F: FnOnce(&Self) -> Result<T, E>,
    {
        with_context(self.input.clone(), context, || f(self))
    }

    /// Read a length of input that was successfully consumed from a sub-parse.
    pub fn take_consumed<F>(&mut self, consumer: F) -> I
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Self),
    {
        self.advance(|input| input.split_consumed(consumer))
    }

    /// Try read a length of input that was successfully consumed from a
    /// sub-parse.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Invalid};
    ///
    /// let result: Result<_, Invalid> = dangerous::input(b"abc").read_all(|r| {
    ///     r.try_take_consumed(|r| {
    ///         r.skip(1)?;
    ///         r.consume(b"bc")
    ///     })
    /// });
    ///
    /// assert_eq!(result.unwrap(), b"abc"[..]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_consumed<F>(&mut self, consumer: F) -> Result<I, E>
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Self) -> Result<(), E>,
    {
        self.try_advance(|input| input.try_split_consumed(consumer, "try take consumed"))
    }

    /// Read and verify a value without returning it.
    ///
    /// # Errors
    ///
    /// Returns an error if the verifier function returned `false`.
    pub fn verify<F>(&mut self, expected: &'static str, verifier: F) -> Result<(), E>
    where
        F: FnOnce(&mut Self) -> bool,
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        self.try_advance(|input| {
            input.split_expect(
                |r: &mut Self| {
                    if verifier(r) {
                        Some(())
                    } else {
                        None
                    }
                },
                expected,
                "verify",
            )
        })
    }

    /// Try read and verify a value without returning it.
    ///
    /// # Errors
    ///
    /// Returns an error if the verifier function returned `false` or an error.
    pub fn try_verify<F>(&mut self, expected: &'static str, verifier: F) -> Result<(), E>
    where
        F: FnOnce(&mut Self) -> Result<bool, E>,
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        self.try_advance(|input| {
            input.try_split_expect(
                |r: &mut Self| match verifier(r) {
                    Ok(true) => Ok(Some(())),
                    Ok(false) => Ok(None),
                    Err(err) => Err(err),
                },
                expected,
                "try verify",
            )
        })
    }

    /// Expect a value to be read and returned as `Some(T)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the returned value was `None`.
    pub fn expect<F, T>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Option<T>,
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
    {
        self.try_advance(|input| input.split_expect(f, expected, "expect"))
    }

    /// Expect a value to be read successfully and returned as `Some(O)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the returned value was `None` or if the provided
    /// function does.
    pub fn try_expect<F, T>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Self) -> Result<Option<T>, E>,
    {
        self.try_advance(|input| input.try_split_expect(f, expected, "try expect"))
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
    /// use dangerous::{BytesReader, Error, Expected, Input, Invalid};
    ///
    /// // Our custom reader function
    /// fn read_ipv4_addr<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Ipv4Addr, E>
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
    #[cfg(feature = "retry")]
    pub fn try_expect_erased<F, T, R>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        F: FnOnce(&mut Self) -> Result<T, R>,
        R: ToRetryRequirement,
    {
        self.try_advance(|input| input.try_split_expect_erased(f, expected, "try expect erased"))
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
        let checkpoint = self.input.clone();
        if let Ok(ok) = f(self) {
            Some(ok)
        } else {
            self.input = checkpoint;
            None
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
        E: WithContext<'i>,
        F: FnOnce(&mut Self) -> Result<T, E>,
        R: FnOnce(&E) -> bool,
    {
        let checkpoint = self.input.clone();
        match f(self) {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => {
                if pred(&err) {
                    self.input = checkpoint;
                    Ok(None)
                } else {
                    Err(err.with_context(checkpoint, OperationContext("recover if")))
                }
            }
        }
    }

    /// Read with a different error type.
    ///
    /// Keep in mind using different errors types can increase your binary size,
    /// use sparingly.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{BytesReader, Error, Expected, Fatal, Input};
    ///
    /// fn branch_a<'i, E>(r: &mut BytesReader<'i, E>) -> Result<u8, E>
    /// where
    ///     E: Error<'i>
    /// {
    ///     r.consume(b"hello").map(|()| 1)
    /// }
    ///
    /// fn branch_b<'i, E>(r: &mut BytesReader<'i, E>) -> Result<u8, E>
    /// where
    ///     E: Error<'i>
    /// {
    ///     r.consume(b"world").map(|()| 2)
    /// }
    ///
    /// let input = dangerous::input(b"world");
    /// let result: Result<_, Expected> = input.read_all(|r| {
    ///     r.expect("valid branch", |r| {
    ///         r.error(|r: &mut BytesReader<Fatal>| {
    ///             r.recover(branch_a).or_else(|| r.recover(branch_b))
    ///         })
    ///     })
    /// });
    ///
    /// assert_eq!(result.unwrap(), 2);
    /// ```
    #[inline]
    pub fn error<F, T, S>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Reader<'i, S, I>) -> T,
    {
        self.advance(|input| {
            let mut sub = Reader::new(input);
            let ok = f(&mut sub);
            (ok, sub.input)
        })
    }

    /// Create a `Reader` given `Input`.
    pub(crate) fn new(input: I) -> Self {
        Self {
            input,
            types: PhantomData,
        }
    }

    #[inline(always)]
    fn advance<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(I) -> (O, I),
    {
        let (ok, next) = f(self.input.clone());
        self.input = next;
        ok
    }

    #[inline(always)]
    fn try_advance<F, SE, O>(&mut self, f: F) -> Result<O, SE>
    where
        F: FnOnce(I) -> Result<(O, I), SE>,
    {
        match f(self.input.clone()) {
            Ok((ok, next)) => {
                self.input = next;
                Ok(ok)
            }
            Err(err) => Err(err),
        }
    }
}

impl<'i, E, I> fmt::Debug for Reader<'i, E, I>
where
    I: Input<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("input", &self.input)
            .finish()
    }
}

#[cfg(feature = "zc")]
unsafe impl<'i, E, I> zc::NoInteriorMut for Reader<'i, E, I>
where
    E: zc::NoInteriorMut,
    I: zc::NoInteriorMut + Input<'i>,
{
}
