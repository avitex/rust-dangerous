mod bytes;
mod input;
mod peek;

use core::marker::PhantomData;

use crate::fmt;
use crate::input::{Bytes, Input, String};

pub use self::peek::Peek;

/// [`Bytes`] specific [`Reader`].
pub type BytesReader<'i, E> = Reader<'i, Bytes<'i>, E>;

/// [`String`] specific [`Reader`].
pub type StringReader<'i, E> = Reader<'i, String<'i>, E>;

/// Created from and consumes an [`Input`].
///
/// You can only create a [`Reader`] from [`Input`] via [`Input::read_all()`],
/// [`Input::read_partial()`] or [`Input::read_infallible()`].
///
/// See [`BytesReader`] for [`Bytes`] specific functions and [`StringReader`]
/// for [`String`] specific functions.
///
/// # Conventions
///
/// - `take_*` functions _take_ [`Input`] from the reader.
/// - `read_*` functions _read_ a primitive type from the reader and take a
///   description of what it is for.
/// - `peek_*` functions _peek_ either [`Input`] or a primitive type from the
///   reader without effecting its internal state.
/// - `try_*` functions accept a function that can return an error as an
///   argument.
///
/// # Errors
///
/// Functions on `Reader` are designed to provide a panic free interface and if
/// applicable, clearly define the type of error that can can be thown.
///
/// To verify input and optionally return a type from that verification,
/// [`verify()`], [`try_verify()`], [`expect()`], [`try_expect()`] and
/// [`try_external()`] is provided. These functions are the interface for
/// creating errors based off what was expected.
///
/// [`try_external()`] is provided for reading a custom type that does
/// not support a `&mut Reader<'i, I, E>` interface, for example a type
/// implementing `FromStr`.
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
///     // We use `peek_read` here because we expect at least one byte.
///     // If we wanted to handle the case when there is no more input left,
///     // for example to provide a default, we would use `peek_read_opt`.
///     // The below allows us to handle a `RetryRequirement` if the
///     // `Reader` is at the end of the input.
///     r.try_expect("boolean", |r| match r.peek_read()? {
///         b't' => r.consume(b"true").map(|()| Some(true)),
///         b'f' => r.consume(b"false").map(|()| Some(false)),
///         _ => Ok(None),
///     })
/// });
/// assert!(matches!(result, Ok(true)));
/// ```
///
/// [`Input`]: crate::Input  
/// [`Context`]: crate::error::Context  
/// [`Input::read_all()`]: crate::Input::read_all()  
/// [`Input::read_partial()`]: crate::Input::read_partial()  
/// [`Input::read_infallible()`]: crate::Input::read_infallible()  
/// [`context()`]: Reader::context()  
/// [`peek_context()`]: Reader::peek_context()  
/// [`verify()`]: Reader::verify()  
/// [`try_verify()`]: Reader::try_verify()  
/// [`expect()`]: Reader::expect()  
/// [`try_expect()`]: Reader::try_expect()  
/// [`try_external()`]: Reader::try_external()  
/// [`recover()`]: Reader::recover()  
/// [`recover_if()`]: Reader::recover_if()  
/// [`RetryRequirement`]: crate::error::RetryRequirement  
pub struct Reader<'i, I, E> {
    input: I,
    types: PhantomData<(&'i (), E)>,
}

impl<'i, I, E> Reader<'i, I, E>
where
    I: Input<'i>,
{
    /// Create a `Reader` given `Input`.
    pub(crate) fn new(input: I) -> Self {
        Self {
            input,
            types: PhantomData,
        }
    }

    /// Advances the reader's input given an operation.
    #[inline(always)]
    fn advance<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(I) -> (O, I),
    {
        let (ok, next) = f(self.input.clone());
        self.input = next;
        ok
    }

    /// Optionally advances the reader's input given an operation.
    #[inline(always)]
    fn advance_opt<F, O>(&mut self, f: F) -> Option<O>
    where
        F: FnOnce(I) -> Option<(O, I)>,
    {
        if let Some((ok, next)) = f(self.input.clone()) {
            self.input = next;
            Some(ok)
        } else {
            None
        }
    }

    /// Tries to advance the reader's input given an operation.
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

impl<'i, I, E> fmt::Debug for Reader<'i, I, E>
where
    I: Input<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reader")
            .field("input", &self.input)
            .finish()
    }
}
