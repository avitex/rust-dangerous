use crate::input::{Bound, Bytes, Input, String};

/// Creates a new `Input` from a byte or string slice.
///
/// It is recommended to use this directly from the crate as `dangerous::input()`,
/// not as an import via `use` as shown below, as you lose the discoverability.
///
/// ```
/// use dangerous::input; // bad
///
/// dangerous::input(b"hello"); // do this instead
/// ```
pub fn input<'i, I>(input: I) -> I::Input
where
    I: IntoInput<'i>,
{
    input.into_input()
}

///////////////////////////////////////////////////////////////////////////////
// Private entry and implementations

pub trait IntoInput<'i>: Copy {
    type Input: Input<'i>;

    fn into_input(self) -> Self::Input;
}

impl<'i, T> IntoInput<'i> for &T
where
    T: IntoInput<'i>,
{
    type Input = T::Input;

    fn into_input(self) -> Self::Input {
        (*self).into_input()
    }
}

impl<'i> IntoInput<'i> for &'i [u8] {
    type Input = Bytes<'i>;

    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}

impl<'i> IntoInput<'i> for &'i str {
    type Input = String<'i>;

    fn into_input(self) -> Self::Input {
        String::new(self, Bound::Start)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Array impl

#[cfg(feature = "const-generics")]
impl<'i, const N: usize> IntoInput<'i> for &'i [u8; N] {
    type Input = Bytes<'i>;

    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}

#[cfg(not(feature = "const-generics"))]
macro_rules! impl_array_into_input {
    ($($n:expr),*) => {
        $(
            impl<'i> IntoInput<'i> for &'i [u8; $n] {
                type Input = Bytes<'i>;

                fn into_input(self) -> Self::Input {
                    Bytes::new(self, Bound::Start)
                }
            }
        )*
    };
}

#[cfg(not(feature = "const-generics"))]
impl_array_into_input!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 32,
    64, 128, 256
);
