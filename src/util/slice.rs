#[inline(always)]
pub(crate) fn is_sub_slice<T>(parent: &[T], sub: &[T]) -> bool {
    let parent_bounds = parent.as_ptr_range();
    let sub_bounds = sub.as_ptr_range();
    parent_bounds.start <= sub_bounds.start && parent_bounds.end >= sub_bounds.end
}

#[inline(always)]
pub(crate) fn end<T>(slice: &[T]) -> &[T] {
    // SAFETY: This is always valid as we a getting a new slice from its own
    // length.
    unsafe { slice.get_unchecked(slice.len()..) }
}

#[inline(always)]
pub(crate) unsafe fn first_unchecked<T: Copy>(slice: &[T]) -> T {
    *slice.get_unchecked(0)
}

#[inline(always)]
pub(crate) fn split_at_opt<T>(slice: &[T], mid: usize) -> Option<(&[T], &[T])> {
    if mid > slice.len() {
        None
    } else {
        // SAFETY: We have checked that 0 <= mid <= slice.len()
        unsafe { Some(split_at_unchecked(slice, mid)) }
    }
}

#[inline(always)]
pub(crate) unsafe fn split_at_unchecked<T>(slice: &[T], mid: usize) -> (&[T], &[T]) {
    // SAFETY: Caller has to check that `0 <= mid <= self.len()`
    (slice.get_unchecked(..mid), slice.get_unchecked(mid..))
}
