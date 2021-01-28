use core::convert::Infallible;

pub(crate) mod alt_iter;
pub(crate) mod slice;
pub(crate) mod utf8;

pub(crate) fn unwrap_ok_infallible<T>(result: Result<T, Infallible>) -> T {
    result.unwrap()
}
