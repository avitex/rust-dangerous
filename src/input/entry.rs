use super::{Bound, Bytes, Input, String};

pub trait IntoInput<'i>: Copy {
    type Input: Input<'i>;

    fn into_input(self) -> Self::Input;
}

impl<'i, T> IntoInput<'i> for &T
where
    T: IntoInput<'i>,
{
    type Input = T::Input;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        (*self).into_input()
    }
}

impl<'i> IntoInput<'i> for &'i [u8] {
    type Input = Bytes<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}

impl<'i> IntoInput<'i> for &'i str {
    type Input = String<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        String::new(self, Bound::Start)
    }
}

impl<'i, const N: usize> IntoInput<'i> for &'i [u8; N] {
    type Input = Bytes<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}
