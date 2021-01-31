use core::marker::PhantomData;
use core::ops::Deref;

use crate::input::Input;

/// Peeked [`Input`].
pub struct Peek<'p, I> {
    input: I,
    _life: PhantomData<&'p ()>,
}

impl<'p, I> Peek<'p, I> {
    #[inline(always)]
    pub(super) fn new(input: I) -> Self {
        Self {
            input,
            _life: PhantomData,
        }
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
