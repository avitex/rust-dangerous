pub(crate) mod alt_iter;
pub(crate) mod byte;
pub(crate) mod utf8;

#[inline(always)]
pub(crate) fn is_sub_slice<T>(parent: &[T], sub: &[T]) -> bool {
    let parent_bounds = parent.as_ptr_range();
    let sub_bounds = sub.as_ptr_range();
    parent_bounds.start <= sub_bounds.start && parent_bounds.end >= sub_bounds.end
}
