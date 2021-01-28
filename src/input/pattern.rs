use super::Bytes;

/// A structure that can be found within an [`Input`](crate::Input).
///
/// # Safety
///
/// The implementation must returned valid indexes and lengths for splitting
/// input as these are not checked.
pub unsafe trait Pattern<I>: Copy {
    /// Returns the index and length of the match if one is found.
    fn find(self, input: &I) -> Option<(usize, usize)>;
}

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    #[cfg(feature = "memchr")]
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        memchr::memchr(self, input.as_dangerous()).map(|index| (index, 1))
    }
    #[cfg(not(feature = "memchr"))]
    fn find(self, input: &Bytes<'i>) -> Option<usize> {
        unimplemented!()
    }
}
