use crate::input::{Input, Pattern, Prefix, PrivateExt};

use crate::error::{
    with_context, Context, CoreContext, CoreOperation, ExpectedLength, ExpectedValid,
    ExpectedValue, External, Value, WithContext,
};

use super::{Peek, Reader};

impl<'i, I, E> Reader<'i, I, E>
where
    I: Input<'i>,
{
    /// Returns `true` if the `Reader` has no more input to consume.
    #[must_use]
    #[inline(always)]
    pub fn at_end(&self) -> bool {
        self.input.is_empty()
    }

    /// Returns the number of input bytes left within the reader.
    ///
    /// This number is subject to change in future passes for streaming input.
    pub fn remaining_bytes(&self) -> usize {
        self.input.byte_len()
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
        with_context(context, self.input.clone(), || f(self))
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
        with_context(context, self.input.clone(), || f(self))
    }

    /// Read a length of input that was successfully consumed from a sub-parse.
    pub fn take_consumed<F, T>(&mut self, consumer: F) -> (T, I)
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.advance(|input| {
            let (value, head, tail) = input.split_consumed(consumer);
            ((value, head), tail)
        })
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
    ///     let ((), consumed) = r.try_take_consumed(|r| {
    ///         r.skip(1)?;
    ///         r.consume(b"bc")?;
    ///         Ok(())
    ///     })?;
    ///     Ok(consumed)
    /// });
    ///
    /// assert_eq!(result.unwrap(), b"abc"[..]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns any error the provided function does.
    pub fn try_take_consumed<F, T>(&mut self, consumer: F) -> Result<(T, I), E>
    where
        E: WithContext<'i>,
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        self.try_advance(|input| {
            input
                .try_split_consumed(consumer, CoreOperation::TakeConsumed)
                .map(|(value, head, tail)| ((value, head), tail))
        })
    }

    /// Read and verify a value without returning it.
    ///
    /// # Errors
    ///
    /// Returns an error if the verifier function returned `false`.
    pub fn verify<F>(&mut self, expected: &'static str, verifier: F) -> Result<(), E>
    where
        F: FnOnce(&mut Self) -> bool,
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
                CoreOperation::Verify,
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
                CoreOperation::Verify,
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
        self.try_advance(|input| input.split_expect(f, expected, CoreOperation::Expect))
    }

    /// Expect a value to be read successfully and returned as `Some(T)`.
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
        self.try_advance(|input| input.try_split_expect(f, expected, CoreOperation::Expect))
    }

    /// Tries to read an expected value with support for an external error.
    ///
    /// This function is useful for reading custom/unsupported types easily
    /// without having to create custom errors.
    ///
    /// # Usage
    ///
    /// On success, the provided function must return `Ok((T, usize))` where `T`
    /// is the type being read and `usize` being the length in bytes of input
    /// that was consumed. The length of bytes returned **MUST** align to a
    /// token boundary within the input, else an error will be returned. For a
    /// [`BytesReader`] this doesn't matter, but for a [`StringReader`] the
    /// length returned must sit on a valid char boundary.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{BytesReader, Error, Expected, Input};
    ///
    /// // Our custom reader function
    /// fn read_number<'i, E>(r: &mut BytesReader<'i, E>) -> Result<u32, E>
    /// where
    ///   E: Error<'i>,
    /// {
    ///     r.take_remaining_str()?.read_all(|r| {
    ///         r.try_expect_external("number", |i| {
    ///             // We map the parsed number along with the byte length
    ///             // of the input to tell the reader we read all of it.
    ///             i.as_dangerous().parse().map(|number| (i.byte_len(), number))
    ///         })
    ///     })
    /// }
    ///
    /// let input = dangerous::input(b"12x");
    /// let error: Expected<'_> = input.read_all(read_number).unwrap_err();
    ///
    /// // Prefer string input formatting
    /// println!("{:#}", error);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if:
    ///
    /// - the provided function returns an amount of input read not aligned to a
    ///   token boundary
    /// - the provided function returns an [`External`] error.
    ///
    /// Returns [`ExpectedLength`] if:
    ///
    /// - the provided function returns an amount of input read that is greater
    ///   than the actual length
    ///
    /// [`BytesReader`]: crate::BytesReader  
    /// [`StringReader`]: crate::StringReader
    pub fn try_expect_external<F, T, R>(&mut self, expected: &'static str, f: F) -> Result<T, E>
    where
        E: WithContext<'i>,
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
        F: FnOnce(I) -> Result<(usize, T), R>,
        R: External<'i>,
    {
        self.try_advance(|input| {
            input.try_split_expect_external(f, expected, CoreOperation::ExpectExternal)
        })
    }

    /// Recovers from an error returning `Some(T)` if successful, or `None` if
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
                    Err(err
                        .with_context(CoreContext::from_operation(
                            CoreOperation::RecoverIf,
                            checkpoint.span(),
                        ))
                        .with_input(checkpoint))
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
    /// let result: Result<_, Expected<'_>> = input.read_all(|r| {
    ///     r.expect("valid branch", |r| {
    ///         r.error(|r: &mut BytesReader<'_, Fatal>| {
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
        F: FnOnce(&mut Reader<'i, I, S>) -> T,
    {
        self.advance(|input| {
            let mut sub = Reader::new(input);
            let ok = f(&mut sub);
            (ok, sub.input)
        })
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
        self.try_advance(|input| input.split_at(len, CoreOperation::Take))
    }

    /// Read an optional length of input.
    ///
    /// Returns `Some(I)` if there was enough input, `None` if not.
    pub fn take_opt(&mut self, len: usize) -> Option<I> {
        self.advance_opt(|input| input.split_at_opt(len))
    }

    /// Read a length of input while a pattern matches.
    ///
    /// Returns the input leading up to when the predicate returns `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Invalid};
    ///
    /// let result: Result<_, Invalid> = dangerous::input(b"hello!").read_all(|r| {
    ///     r.take_while(|b: u8| b.is_ascii_alphabetic());
    ///     r.consume(b'!')
    /// });
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn take_while<P>(&mut self, pattern: P) -> I
    where
        P: Pattern<I>,
    {
        self.advance(|input| match input.clone().split_while_opt(pattern) {
            Some((taken, next)) => (taken, next),
            None => (input.clone(), input.end()),
        })
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
        self.try_advance(|input| input.try_split_while(pred, CoreOperation::TakeWhile))
    }

    /// Read a length of input until a expected pattern matches.
    ///
    /// Returns the input leading up to the pattern match.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Invalid};
    ///
    /// let result: Result<_, Invalid> = dangerous::input(b"hello world").read_all(|r| {
    ///     let before_space = r.take_until(b' ')?;
    ///     Ok((before_space, r.take_remaining()))
    /// });
    ///
    /// let (before_space, remaining) = result.unwrap();
    ///
    /// assert_eq!(before_space, b"hello"[..]);
    /// assert_eq!(remaining, b" world"[..]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValue`] if the pattern could not be found.
    pub fn take_until<P>(&mut self, pattern: P) -> Result<I, E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<I> + Into<Value<'i>> + Copy,
    {
        self.try_advance(|input| input.split_until(pattern, CoreOperation::TakeUntil))
    }

    /// Read a length of input until a pattern matches and consumes the matched
    /// input if found.
    ///
    /// Returns the input leading up to the pattern match.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Invalid};
    ///
    /// let result: Result<_, Invalid> = dangerous::input(b"hello world").read_all(|r| {
    ///     let before_space = r.take_until_consume(b' ')?;
    ///     Ok((before_space, r.take_remaining()))
    /// });
    ///
    /// let (before_space, remaining) = result.unwrap();
    ///
    /// assert_eq!(before_space, b"hello"[..]);
    /// assert_eq!(remaining, b"world"[..]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValue`] if the pattern could not be found.
    pub fn take_until_consume<P>(&mut self, pattern: P) -> Result<I, E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<I> + Into<Value<'i>> + Copy,
    {
        self.try_advance(|input| {
            input.split_until_consume(pattern, CoreOperation::TakeUntilConsume)
        })
    }

    /// Read a length of input until a pattern optionally matches.
    ///
    /// If you want to know whether the pattern was consumed or not, check
    /// [`Reader::at_end()`]. If the reader is at the end, then the pattern was
    /// not consumed.
    ///
    /// Returns the input leading up to the pattern match if any.
    pub fn take_until_opt<P>(&mut self, pattern: P) -> I
    where
        P: Pattern<I>,
    {
        self.advance(|input| match input.clone().split_until_opt(pattern) {
            Some((taken, next)) => (taken, next),
            None => (input.clone(), input.end()),
        })
    }

    /// Read a length of input until a pattern optionally matches and consumes
    /// the matched input if found.
    ///
    /// Returns a tuple with:
    ///
    /// - The input leading up to the pattern match if any.
    /// - `true` if the pattern was consumed, `false` if not.
    pub fn take_until_consume_opt<P>(&mut self, pattern: P) -> (I, bool)
    where
        P: Pattern<I>,
    {
        self.advance(
            |input| match input.clone().split_until_consume_opt(pattern) {
                Some((taken, next)) => ((taken, true), next),
                None => ((input.clone(), false), input.end()),
            },
        )
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
        self.try_advance(|input| input.split_at(len, CoreOperation::Skip))
            .map(drop)
    }

    /// Skip an optional length of input.
    ///
    /// Returns `true` if there was enough input, `false` if not.
    pub fn skip_opt(&mut self, len: usize) -> bool {
        self.advance_opt(|input| input.split_at_opt(len)).is_some()
    }

    /// Skip a length of input while a pattern matches.
    pub fn skip_while<P>(&mut self, pattern: P)
    where
        P: Pattern<I>,
    {
        let _skipped = self.take_while(pattern);
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
        self.try_advance(|input| input.try_split_while(pred, CoreOperation::SkipWhile))
            .map(drop)
    }

    /// Skip a length of input until a expected pattern matches.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValue`] if the pattern could not be found.
    pub fn skip_until<P>(&mut self, pattern: P) -> Result<(), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<I> + Into<Value<'i>> + Copy,
    {
        self.try_advance(|input| input.split_until(pattern, CoreOperation::SkipUntil))
            .map(drop)
    }

    /// Skip a length of input until a pattern matches and consumes the matched
    /// input if found.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValue`] if the pattern could not be found.
    pub fn skip_until_consume<P>(&mut self, pattern: P) -> Result<(), E>
    where
        E: From<ExpectedValue<'i>>,
        P: Pattern<I> + Into<Value<'i>> + Copy,
    {
        self.try_advance(|input| {
            input.split_until_consume(pattern, CoreOperation::SkipUntilConsume)
        })
        .map(drop)
    }

    /// Skip a length of input until a pattern optionally matches.
    pub fn skip_until_opt<P>(&mut self, pattern: P)
    where
        P: Pattern<I>,
    {
        let _skipped = self.take_until_opt(pattern);
    }

    /// Skip a length of input until a pattern optionally matches and consumes
    /// the matched input if found.
    ///
    /// Returns `true` if the pattern was consumed, `false` if not.
    pub fn skip_until_consume_opt<P>(&mut self, pattern: P) -> bool
    where
        P: Pattern<I>,
    {
        let (_skipped, consumed) = self.take_until_consume_opt(pattern);
        consumed
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
        self.try_advance(|input| input.split_prefix(prefix, CoreOperation::Consume))
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

    /// Peek a length of input.
    ///
    /// The function lifetime `'p` helps prevent the peeked [`Input`] being used
    /// as a value in a parsed structure. Peeked values should only be used in
    /// choosing a correct parse path.
    ///
    /// # Errors
    ///
    /// Returns an error if the length requirement to peek could not be met.
    pub fn peek<'p>(&'p self, len: usize) -> Result<Peek<'p, I>, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        match self.input.clone().split_at(len, CoreOperation::Peek) {
            Ok((head, _)) => Ok(Peek::new(head)),
            Err(err) => Err(err),
        }
    }

    /// Peek a length of input.
    ///
    /// This is equivalent to `peek` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[must_use = "peek result must be used"]
    #[allow(clippy::needless_lifetimes)]
    pub fn peek_opt<'p>(&'p self, len: usize) -> Option<Peek<'p, I>> {
        self.input
            .clone()
            .split_at_opt(len)
            .map(|(head, _)| Peek::new(head))
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
}
