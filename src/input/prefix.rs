use super::ByteLength;

/// Implemented for types that can be a prefix for a given input.
/// 
/// # Safety
///
/// The implementation **must** guarantee that the value returned is correct as
/// it is used for unchecked memory operations and an incorrect implementation
/// would introduce invalid memory access.
pub unsafe trait Prefix<I>: ByteLength + Copy {
    /// Returns `true` if `self` is a prefix of the given input.
    fn is_prefix_of(self, input: &I) -> bool;
}

unsafe impl<'i, T, I> Prefix<I> for &T
where
    T: Prefix<I>,
{
    #[inline(always)]
    fn is_prefix_of(self, input: &I) -> bool {
        (*self).is_prefix_of(input)
    }
}
