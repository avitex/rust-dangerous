mod bytes;
mod maybe;
mod string;
mod traits;

pub use self::bytes::Bytes;
pub use self::maybe::MaybeString;
pub use self::string::String;
pub use self::traits::Input;

pub(crate) use self::traits::{Private, PrivateExt};

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
#[inline(always)]
pub fn input<'i, I>(input: I) -> I::Input
where
    I: IntoInput<'i>,
{
    input.into_input()
}

///////////////////////////////////////////////////////////////////////////////
// Bound

/// Indication of whether [`Input`] will change in futher passes.
///
/// Used for retry functionality if enabled.
#[must_use]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Bound {
    /// Both sides of the [`Input`](crate::Input) may change in further passes.
    None,
    /// The start of the [`Input`](crate::Input) in further passes will not change.
    ///
    /// The end of the [`Input`](crate::Input) may however change in further passes.
    Start,
    /// Both sides of the [`Input`](crate::Input) in further passes will not change.
    Both,
}

impl Bound {
    #[inline(always)]
    pub(crate) fn force_close() -> Self {
        Bound::Both
    }

    /// An end is opened when it is detected a `take_consumed` reader could have
    /// continued.
    #[inline(always)]
    pub(crate) fn open_end(self) -> Self {
        match self {
            // If at least the start is bound make sure the end is unbound.
            Bound::Both | Bound::Start => Bound::Start,
            // If the start is unbound both sides of the input are unbound.
            Bound::None => Bound::None,
        }
    }

    /// An end is closed when a known length of input is sucessfully taken.
    #[inline(always)]
    pub(crate) fn close_end(self) -> Self {
        // We don't care if the input has no bounds. The only place input with
        // no bounds can originate is when a reader has reached the end of input
        // and could have consumed more. In other words - input with no bounds
        // is always empty. A length of zero taken from input with no bounds
        // will always succeed but the first half will have both sides bound to
        // prevent deadlocks.
        let _ = self;
        Bound::force_close()
    }

    #[inline(always)]
    pub(crate) fn for_end(self) -> Self {
        match self {
            // If both sides are bounded nothing will change.
            Bound::Both => Bound::Both,
            // As we have skipped to the end without checking, we don't know
            // where the start is, perhaps the true end is not known yet!
            Bound::Start | Bound::None => Bound::None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IntoInput

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

///////////////////////////////////////////////////////////////////////////////
// Array impl

#[cfg(feature = "unstable-const-generics")]
impl<'i, const N: usize> IntoInput<'i> for &'i [u8; N] {
    type Input = Bytes<'i>;

    #[inline(always)]
    fn into_input(self) -> Self::Input {
        Bytes::new(self, Bound::Start)
    }
}

#[cfg(not(feature = "unstable-const-generics"))]
macro_rules! impl_array_into_input {
    ($($n:expr),*) => {
        $(
            impl<'i> IntoInput<'i> for &'i [u8; $n] {
                type Input = Bytes<'i>;

                #[inline(always)]
                fn into_input(self) -> Self::Input {
                    Bytes::new(self, Bound::Start)
                }
            }
        )*
    };
}

#[cfg(not(feature = "unstable-const-generics"))]
impl_array_into_input!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 32,
    64, 128, 256
);
