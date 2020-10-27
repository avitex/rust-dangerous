pub(crate) mod alt_iter;
pub(crate) mod byte;
pub(crate) mod utf8;

use core::ops::Range;

// FIXME: use https://github.com/rust-lang/rust/issues/65807 when stable in 1.48
#[inline(always)]
pub(crate) fn slice_ptr_range<T>(slice: &[T]) -> Range<*const T> {
    let start = slice.as_ptr();
    // note: will never wrap, we are just escaping the use of unsafe.
    let end = slice.as_ptr().wrapping_add(slice.len());
    debug_assert!(start <= end);
    start..end
}

#[inline(always)]
pub(crate) fn is_sub_slice<T>(parent: &[T], sub: &[T]) -> bool {
    let parent_bounds = slice_ptr_range(parent);
    let sub_bounds = slice_ptr_range(sub);
    parent_bounds.start <= sub_bounds.start && parent_bounds.end >= sub_bounds.end
}
