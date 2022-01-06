use crate::display::InputDisplay;
use crate::{fmt, Bound, Bytes, Input, Span};

/// Fixed size byte array taken from [`Input`].
#[derive(Clone)]
pub struct ByteArray<'i, const N: usize>(&'i [u8; N]);

impl<'i, const N: usize> ByteArray<'i, N> {
    pub(crate) const fn new(bytes: &'i [u8; N]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying byte array reference.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    #[must_use]
    #[inline(always)]
    pub fn as_dangerous(&self) -> &'i [u8; N] {
        self.0
    }

    /// Returns the underlying byte array.
    ///
    /// This will copy the bytes from the reference.
    ///
    /// See [`Bytes::as_dangerous`] for naming.
    #[must_use]
    #[inline(always)]
    pub fn into_dangerous(self) -> [u8; N] {
        *self.as_dangerous()
    }

    /// Consumes `self` into [`Bytes`].
    #[inline(always)]
    pub fn into_bytes(self) -> Bytes<'i> {
        Bytes::new(self.as_dangerous(), Bound::StartEnd)
    }

    /// Returns a [`Span`] from the start of `self` to the end.
    pub fn span(&self) -> Span {
        Span::from(self.as_dangerous().as_ref())
    }

    /// Returns an [`InputDisplay`] for formatting.
    pub fn display(&self) -> InputDisplay<'i> {
        self.clone().into_bytes().display()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Equality

impl<'i, const N: usize> PartialEq for ByteArray<'i, N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_dangerous() == other.as_dangerous()
    }
}

impl<'i, const N: usize> PartialEq<[u8]> for ByteArray<'i, N> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i, const N: usize> PartialEq<[u8]> for &ByteArray<'i, N> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl<'i, const N: usize> PartialEq<&[u8]> for ByteArray<'i, N> {
    #[inline(always)]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_dangerous() == *other
    }
}

impl<'i, const N: usize> PartialEq<ByteArray<'i, N>> for [u8] {
    #[inline(always)]
    fn eq(&self, other: &ByteArray<'i, N>) -> bool {
        self == other.as_dangerous()
    }
}

impl<'i, const N: usize> PartialEq<Bytes<'i>> for ByteArray<'i, N> {
    #[inline(always)]
    fn eq(&self, other: &Bytes<'i>) -> bool {
        self == other.as_dangerous()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl<'i, const N: usize> fmt::Debug for ByteArray<'i, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = self.display().with_formatter(f);
        f.debug_tuple("ByteArray").field(&display).finish()
    }
}

impl<'i, const N: usize> fmt::DisplayBase for ByteArray<'i, N> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.display().fmt(w)
    }
}

impl<'i, const N: usize> fmt::Display for ByteArray<'i, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display().with_formatter(f).fmt(f)
    }
}
