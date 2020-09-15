use core::fmt;

use crate::error::{Context, Error, OperationContext};
use crate::input::Input;

#[inline(always)]
pub(crate) fn with_operation_context<'i, F, T, E>(
    input: &'i Input,
    operation: &'static str,
    f: F,
) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: Error<'i>,
{
    with_context(input, OperationContext(operation), f)
}

#[inline(always)]
pub(crate) fn with_context<'i, F, C, T, E>(input: &'i Input, context: C, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: Error<'i>,
    C: Context,
{
    f().map_err(|err| err.from_context(input, context))
}

pub(crate) struct ByteCount(pub(crate) usize);

impl fmt::Display for ByteCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => f.write_str("no bytes"),
            1 => f.write_str("1 byte"),
            n => write!(f, "{} bytes", n),
        }
    }
}

pub(crate) struct WithFormatter<T>(pub(crate) T)
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<T> fmt::Display for WithFormatter<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}
