use core::slice;

use crate::display::InputDisplay;
use crate::fmt;
use crate::input::{Bound, Bytes, Input};
use crate::util::utf8;

/// Value that was expected in an operation.
#[derive(Copy, Clone)]
#[must_use]
pub struct Value<'i>(ValueInner<'i>);

#[derive(Copy, Clone)]
enum ValueInner<'i> {
    Byte(u8),
    Char([u8; 4]),
    Bytes(&'i [u8]),
    String(&'i str),
}

impl<'i> Value<'i> {
    /// Returns the value as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            ValueInner::Byte(v) => slice::from_ref(v),
            ValueInner::Char(v) => &v[..utf8::char_len(v[0])],
            ValueInner::Bytes(v) => v,
            ValueInner::String(v) => v.as_bytes(),
        }
    }

    /// Returns an [`InputDisplay`] for formatting.
    pub fn display(&self) -> InputDisplay<'_> {
        let display = Bytes::new(self.as_bytes(), Bound::Both).display();
        match self.0 {
            ValueInner::Byte(_) | ValueInner::Bytes(_) => display,
            ValueInner::Char(_) | ValueInner::String(_) => display.str_hint(true),
        }
    }
}

impl<'i> fmt::Debug for Value<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            ValueInner::Byte(_) => "Byte",
            ValueInner::Char(_) => "Char",
            ValueInner::Bytes(_) => "Bytes",
            ValueInner::String(_) => "String",
        };
        let display = self.display().str_hint(f.alternate());
        f.debug_tuple(name).field(&display).finish()
    }
}

impl<'i> From<u8> for Value<'i> {
    fn from(v: u8) -> Self {
        Self(ValueInner::Byte(v))
    }
}

impl<'i> From<char> for Value<'i> {
    fn from(v: char) -> Self {
        let mut arr = [0_u8; 4];
        v.encode_utf8(&mut arr);
        Self(ValueInner::Char(arr))
    }
}

impl<'i> From<&'i [u8]> for Value<'i> {
    #[inline(always)]
    fn from(v: &'i [u8]) -> Self {
        Self(ValueInner::Bytes(v))
    }
}

impl<'i> From<&'i str> for Value<'i> {
    #[inline(always)]
    fn from(v: &'i str) -> Self {
        Self(ValueInner::String(v))
    }
}

#[cfg(feature = "unstable-const-generics")]
impl<'i, const N: usize> From<&'i [u8; N]> for Value<'i> {
    #[inline(always)]
    fn from(v: &'i [u8; N]) -> Self {
        Self(ValueInner::Bytes(v))
    }
}

#[cfg(not(feature = "unstable-const-generics"))]
macro_rules! impl_array_into_value {
    ($($n:expr),*) => {
        $(
            impl<'i> From<&'i [u8; $n]> for Value<'i> {
                #[inline(always)]
                fn from(v: &'i [u8; $n]) -> Self {
                    Self(ValueInner::Bytes(v))
                }
            }
        )*
    };
}

#[cfg(not(feature = "unstable-const-generics"))]
for_common_array_sizes!(impl_array_into_value);
