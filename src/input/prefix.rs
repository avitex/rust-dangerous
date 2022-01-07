use super::ByteLength;

/// Implemented for types that can be a prefix for a given input.
///
/// # Safety
///
/// The implementation **must** guarantee that the value returned is correct as
/// it is used for unchecked memory operations and an incorrect implementation
/// would introduce invalid memory access.
pub unsafe trait Prefix<I>: ByteLength {
    /// Returns `true` if `self` is a prefix of the given input.
    fn is_prefix_of(&self, input: &I) -> bool;
}

unsafe impl<'i, T: ?Sized, I> Prefix<I> for &T
where
    T: Prefix<I>,
{
    #[inline(always)]
    fn is_prefix_of(&self, input: &I) -> bool {
        T::is_prefix_of(self, input)
    }
}
