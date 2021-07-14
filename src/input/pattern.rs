/// Implemented for structures that can be found within an
/// [`Input`](crate::Input).
///
/// You can search for a `char` or `&str` within either `Bytes` or `String`, but
/// only a `u8` and `&[u8]` within `Bytes`.
///
/// Empty slices are invalid patterns and have the following behaviour:
///
/// - Finding a match of a empty slice pattern will return `None`.
/// - Finding a reject of a empty slice pattern will return `Some(0)`.
///
/// With the `simd` feature enabled pattern searches are SIMD optimised where
/// possible.
///
/// With the `regex` feature enabled, you can search for regex patterns.
///
/// # Safety
///
/// The implementation must return valid indexes and lengths for splitting input
/// as these are not checked.
pub unsafe trait Pattern<I> {
    /// Returns the byte index and byte length of the first match and `None` if
    /// there was no match.
    fn find_match(self, input: &I) -> Option<(usize, usize)>;

    /// Returns the byte index of the first reject and `None` if there was no
    /// reject.
    fn find_reject(self, input: &I) -> Option<usize>;
}
