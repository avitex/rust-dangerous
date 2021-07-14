/// Implemented for types that return a guaranteed byte length.
///
/// # Safety
///
/// The implementation **must** guarantee that the value returned is correct as
/// it is used for unchecked memory operations and an incorrect implementation
/// would introduce invalid memory access.
pub unsafe trait ByteLength {
    /// Returns the byte length of the value.
    fn byte_len(&self) -> usize;
}

unsafe impl<T> ByteLength for &T
where
    T: ByteLength,
{
    #[inline(always)]
    fn byte_len(&self) -> usize {
        (*self).byte_len()
    }
}

unsafe impl ByteLength for u8 {
    /// Always returns `1`.
    #[inline(always)]
    fn byte_len(&self) -> usize {
        1
    }
}

unsafe impl ByteLength for char {
    /// Returns the length of bytes for the UTF-8 character.
    #[inline(always)]
    fn byte_len(&self) -> usize {
        self.len_utf8()
    }
}

unsafe impl ByteLength for &[u8] {
    /// Returns the length of the byte slice.
    #[inline(always)]
    fn byte_len(&self) -> usize {
        self.len()
    }
}

unsafe impl ByteLength for &str {
    /// Returns the length of the underlying byte slice for the UTF-8 string.
    #[inline(always)]
    fn byte_len(&self) -> usize {
        self.len()
    }
}

unsafe impl<const N: usize> ByteLength for &[u8; N] {
    /// Returns the length (`N`) of the byte array.
    #[inline(always)]
    fn byte_len(&self) -> usize {
        N
    }
}
