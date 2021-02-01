use crate::input::{Bytes, String};

/// A structure that can be found within an [`Input`](crate::Input).
///
/// # Safety
///
/// The implementation must returned valid indexes and lengths for splitting
/// input as these are not checked.
pub unsafe trait Pattern<I>: Copy {
    /// Returns the start byte index and byte length of the match if one is found.
    fn find(self, input: &I) -> Option<(usize, usize)>;
}

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    #[cfg(feature = "memchr")]
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        memchr::memchr(self, input.as_dangerous()).map(|index| (index, 1))
    }
    #[cfg(not(feature = "memchr"))]
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .position(|b| b == self)
            .map(|index| (index, 1))
    }
}

unsafe impl<'i> Pattern<String<'i>> for char {
    // FIXME: add SIMD optimised search
    fn find(self, input: &String<'i>) -> Option<(usize, usize)> {
        input.as_dangerous().char_indices().find_map(|(i, c)| {
            if c == self {
                Some((i, c.len_utf8()))
            } else {
                None
            }
        })
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Matches the start of a pattern without spanning any input.
#[derive(Copy, Clone)]
pub(crate) struct Start<P>(pub(crate) P);

unsafe impl<P, I> Pattern<I> for Start<P>
where
    P: Pattern<I>,
{
    #[inline(always)]
    fn find(self, input: &I) -> Option<(usize, usize)> {
        self.0.find(input).map(|(start, _)| (start, 0))
    }
}
