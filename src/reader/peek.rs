use core::marker::PhantomData;
use core::ops::Deref;

use crate::input::Input;

/// Peek of [`Input`].
///
/// Below is an example of what this structure prevents:
///
/// ```compile_fail
/// use dangerous::{BytesReader, Error};
///
/// fn parse<'i, E>(r: &mut BytesReader<'i, E>) -> Result<&'i [u8], E>
/// where
///    E: Error<'i>
/// {
///     let peeked = r.peek(2)?;
///     Ok(peeked.as_dangerous())
/// }
/// ```
pub struct Peek<'p, I> {
    input: I,
    lifetime: PhantomData<&'p ()>,
}

impl<'p, I> Peek<'p, I> {
    #[inline(always)]
    pub(super) fn new(input: I) -> Self {
        Self {
            input,
            lifetime: PhantomData,
        }
    }

    /// An escape hatch to persist the input beyond the peek lifetime `'p`,
    /// instead falling back to the lifetime associated with `I`.
    #[inline(always)]
    pub fn persist(self) -> I {
        self.input
    }
}

impl<'p, I> AsRef<I> for Peek<'p, I>
where
    I: Input<'p>,
{
    #[inline(always)]
    fn as_ref(&self) -> &I {
        &self.input
    }
}

impl<'p, I> Deref for Peek<'p, I>
where
    I: Input<'p>,
{
    type Target = I;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
