use crate::input::{Input, Prefix, PrivateExt};

#[cfg(feature = "retry")]
use crate::error::ToRetryRequirement;
use crate::error::{
    with_context, Context, ExpectedLength, ExpectedValid, ExpectedValue, OperationContext, Value,
    WithContext,
};

use super::Reader;

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
    #[cfg_attr(docsrs, doc(cfg(feature = "retry")))]
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

    /// Skip `len` number of tokens.
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
    pub fn skip_while<F>(&mut self, pred: F)
    where
        F: FnMut(I::Token) -> bool,
    {
        let _skipped = self.advance(|input| input.split_while(pred));
    }

    /// Try skip a length of input while a predicate check remains successful
    /// and true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_skip_while<F>(&mut self, pred: F) -> Result<(), E>
    where
        E: WithContext<'i>,
        F: FnMut(I::Token) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_while(pred, "try skip while"))
            .map(drop)
    }

    /// Read a length of input.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to read could not be met.
    pub fn take(&mut self, len: usize) -> Result<I, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_at(len, "take"))
    }

    /// Read a length of input while a predicate check remains true.
    pub fn take_while<F>(&mut self, pred: F) -> I
    where
        F: FnMut(I::Token) -> bool,
    {
        self.advance(|input| input.split_while(pred))
    }

    /// Try read a length of input while a predicate check remains successful
    /// and true.
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_while<F>(&mut self, pred: F) -> Result<I, E>
    where
        E: WithContext<'i>,
        F: FnMut(I::Token) -> Result<bool, E>,
    {
        self.try_advance(|input| input.try_split_while(pred, "try take while"))
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
    pub fn peek<'p>(&'p self, len: usize) -> Result<I, E>
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
    pub fn peek_opt(&self, len: usize) -> Option<I> {
        self.input.clone().split_at_opt(len).map(|(head, _)| head)
    }

    /// Returns `true` if `prefix` is next in the `Reader`.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_eq<P>(&self, prefix: P) -> bool
    where
        P: Prefix<I>,
    {
        prefix.is_prefix_of(&self.input)
    }

    /// Consume expected input.
    ///
    /// Doesn't effect the internal state of the `Reader` if the input couldn't
    /// be consumed.
    ///
    /// # Errors
    ///
    /// Returns an error if the input could not be consumed.
    pub fn consume<P>(&mut self, prefix: P) -> Result<(), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Prefix<I> + Into<Value<'i>>,
    {
        self.try_advance(|input| input.split_prefix(prefix, "consume"))
            .map(drop)
    }

    /// Consume optional input.
    ///
    /// Returns `true` if the input was consumed, `false` if not.
    ///
    /// Doesn't effect the internal state of the `Reader` if the input couldn't
    /// be consumed.
    pub fn consume_opt<P>(&mut self, prefix: P) -> bool
    where
        P: Prefix<I>,
    {
        self.advance(|input| {
            let (prefix, next) = input.split_prefix_opt(prefix);
            (prefix.is_some(), next)
        })
    }
}
