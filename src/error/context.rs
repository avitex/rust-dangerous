use core::any::Any;

use crate::fmt;
use crate::input::Input;

use super::WithContext;

/// Information surrounding an error.
pub trait Context: Any {
    /// The operation that was attempted when an error occurred.
    ///
    /// It should described in a simple manner what is trying to be achieved and
    /// make sense in the following sentence if you were to substitute it:
    ///
    /// ```text
    /// error attempting to <operation>.
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result;

    /// Returns `true` if there is an expected value.
    fn has_expected(&self) -> bool {
        false
    }

    /// The expected value.
    ///
    /// # Errors
    ///
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn expected(&self, _w: &mut dyn fmt::Write) -> fmt::Result {
        Err(fmt::Error)
    }

    /// Return a reference of self as [`Any`].
    // FIXME: an ideal implementation wouldn't require this function and we
    // would just lean on the super trait requirement, but doesn't seem possible
    // today with trait objects.
    //
    // See: https://github.com/rust-lang/rfcs/issues/2035
    fn as_any(&self) -> &dyn Any;
}

///////////////////////////////////////////////////////////////////////////////
// Basic expected context

impl Context for &'static str {
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("read")
    }

    fn has_expected(&self) -> bool {
        true
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Expected context

/// A sealed expected [`Context`].
#[derive(Copy, Clone)]
pub struct ExpectedContext {
    pub(crate) operation: &'static str,
    pub(crate) expected: &'static str,
}

impl Context for ExpectedContext {
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self.operation)
    }

    fn has_expected(&self) -> bool {
        !self.expected.is_empty()
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self.expected)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for ExpectedContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedContext")
            .field("operation", &self.operation)
            .field("expected", &self.expected)
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Operation context

/// A sealed operation context.
#[derive(Copy, Clone)]
pub struct OperationContext(pub(crate) &'static str);

impl Context for OperationContext {
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self.0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for OperationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OperationContext").field(&self.0).finish()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[inline(always)]
pub(crate) fn with_context<'i, F, T, E>(
    input: impl Input<'i>,
    context: impl Context,
    f: F,
) -> Result<T, E>
where
    E: WithContext<'i>,
    F: FnOnce() -> Result<T, E>,
{
    match f() {
        Ok(ok) => Ok(ok),
        Err(err) => Err(err.with_context(input, context)),
    }
}
