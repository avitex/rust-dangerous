/// Returns an end slice of the slice provided (always empty).
#[inline(always)]
pub(crate) fn end<T>(slice: &[T]) -> &[T] {
    // SAFETY: This is always valid as we a getting a new slice from its own
    // length.
    unsafe { slice.get_unchecked(slice.len()..) }
}

/// Splits a slice at `mid`.
///
/// Returns `Some` if `0 <= mid <= slice.len()` and `None` otherwise.
#[inline(always)]
pub(crate) fn split_at_opt<T>(slice: &[T], mid: usize) -> Option<(&[T], &[T])> {
    if mid > slice.len() {
        None
    } else {
        // SAFETY: We have checked that 0 <= mid <= slice.len()
        unsafe { Some(split_at_unchecked(slice, mid)) }
    }
}

/// Returns the first item in a slice without bounds checking.
#[inline(always)]
pub(crate) unsafe fn first_unchecked<T: Copy>(slice: &[T]) -> T {
    debug_assert!(!slice.is_empty());
    *slice.get_unchecked(0)
}

/// Splits a slice at `mid` without bounds checking.
///
/// # Safety
///
/// Caller has to check that `0 <= mid <= slice.len()`
#[inline(always)]
pub(crate) unsafe fn split_at_unchecked<T>(slice: &[T], mid: usize) -> (&[T], &[T]) {
    debug_assert!(mid <= slice.len());
    (slice.get_unchecked(..mid), slice.get_unchecked(mid..))
}

/// Splits a str slice at `mid` without bounds checking.
///
/// # Safety
///
/// Caller has to check that `0 <= mid <= slice.len()` and that `mid` is a valid
/// char boundary.
#[inline(always)]
pub(crate) unsafe fn split_str_at_unchecked(slice: &str, mid: usize) -> (&str, &str) {
    debug_assert!(mid <= slice.len());
    debug_assert!(slice.is_char_boundary(mid));
    (slice.get_unchecked(..mid), slice.get_unchecked(mid..))
}

/// Returns the slice as a reference to an array.
///
/// # Safety
///
/// Caller has to check that `slice.len() == N`.
#[inline(always)]
pub(crate) unsafe fn slice_to_array_unchecked<T, const N: usize>(slice: &[T]) -> &[T; N] {
    debug_assert!(slice.len() == N);
    // Cast the slice pointer to an array pointer and reborrow.
    &*slice.as_ptr().cast::<[T; N]>()
}
