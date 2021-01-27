pub(crate) mod alt_iter;
pub(crate) mod slice;
pub(crate) mod utf8;

pub(crate) unsafe fn unwrap_ok_unchecked<T, E>(result: Result<T, E>) -> T {
    debug_assert!(result.is_ok());
    result.unwrap_or_else(|_| core::hint::unreachable_unchecked())
}
