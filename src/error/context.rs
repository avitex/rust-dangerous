use core::any::Any;

use crate::fmt;
use crate::input::Input;

use super::WithContext;

#[cfg(feature = "full-context")]
use alloc::{boxed::Box, vec::Vec};

/// The base context surrounding an error.
pub trait Context: Any {
    /// The operation that was attempted when an error occurred.
    ///
    /// It should described in a simple manner what is trying to be achieved and
    /// make sense in the following sentence if you were to substitute it:
    ///
    /// ```text
    /// error attempting to <operation>.
    /// ```
    fn operation(&self) -> &'static str;

    /// Returns `true` if there is an expected value.
    fn has_expected(&self) -> bool;

    /// The expected value.
    ///
    /// # Errors
    ///
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result;

    /// Return a reference of self as [`Any`].
    // FIXME: an ideal implementation wouldn't require this function and we
    // would just lean on the super trait requirement, but doesn't seem possible
    // today with trait objects.
    //
    // See: https://github.com/rust-lang/rfcs/issues/2035
    fn as_any(&self) -> &dyn Any;
}

/// A walkable stack of [`Context`]s collected from an error.
pub trait ContextStack: 'static {
    /// The root context.
    fn root(&self) -> ExpectedContext;

    /// Return the total number of contexts.
    fn count(&self) -> usize;

    /// Walk the context stack, starting with the highest context to the root.
    ///
    /// Returns `true` if all of the stack available was walked, `false` if not.
    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool;
}

/// A [`ContextStack`] builder.
pub trait ContextStackBuilder {
    /// Create the builder from a root expected context.
    fn from_root(context: ExpectedContext) -> Self;

    /// Push an additional context onto the stack.
    fn push<C>(&mut self, context: C)
    where
        C: Context;
}

/// A dynamic function for walking a context stack.
///
/// Returns `true` if the walk should continue, `false` if not.
///
/// # Parameters
///
/// - `index` (the index of the context starting from `1`).
/// - `context` (the context at the provided index).
pub type ContextStackWalker<'a> = dyn FnMut(usize, &dyn Context) -> bool + 'a;

///////////////////////////////////////////////////////////////////////////////
// Basic expected context

impl Context for &'static str {
    fn operation(&self) -> &'static str {
        "read"
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
    fn operation(&self) -> &'static str {
        self.operation
    }

    fn has_expected(&self) -> bool {
        true
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

#[cfg(feature = "zc")]
unsafe impl zc::NoInteriorMut for ExpectedContext {}

///////////////////////////////////////////////////////////////////////////////
// Operation context

#[derive(Copy, Clone)]
pub(crate) struct OperationContext(pub(crate) &'static str);

impl Context for OperationContext {
    fn operation(&self) -> &'static str {
        self.0
    }

    fn has_expected(&self) -> bool {
        false
    }

    fn expected(&self, _: &mut dyn fmt::Write) -> fmt::Result {
        Err(fmt::Error)
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

#[cfg(feature = "zc")]
unsafe impl zc::NoInteriorMut for OperationContext {}

///////////////////////////////////////////////////////////////////////////////
// Root context stack

/// A [`ContextStack`] that only contains the root [`ExpectedContext`].
pub struct RootContextStack {
    context: ExpectedContext,
}

impl ContextStackBuilder for RootContextStack {
    fn from_root(context: ExpectedContext) -> Self {
        Self { context }
    }

    fn push<C>(&mut self, _context: C)
    where
        C: Context,
    {
    }
}

impl ContextStack for RootContextStack {
    fn root(&self) -> ExpectedContext {
        self.context
    }

    fn count(&self) -> usize {
        1
    }

    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool {
        f(1, &self.context)
    }
}

#[cfg(feature = "zc")]
unsafe impl zc::NoInteriorMut for RootContextStack {}

///////////////////////////////////////////////////////////////////////////////
// Full context stack

/// A [`ContextStack`] that contains all [`Context`]s collected.
#[cfg(feature = "full-context")]
#[cfg_attr(docsrs, doc(cfg(feature = "full-context")))]
pub struct FullContextStack {
    root: ExpectedContext,
    stack: Vec<Box<dyn Context>>,
}

#[cfg(feature = "full-context")]
impl ContextStackBuilder for FullContextStack {
    fn from_root(context: ExpectedContext) -> Self {
        Self {
            root: context,
            stack: Vec::with_capacity(32),
        }
    }

    fn push<C>(&mut self, context: C)
    where
        C: Context,
    {
        self.stack.push(Box::new(context))
    }
}

#[cfg(feature = "full-context")]
impl ContextStack for FullContextStack {
    fn root(&self) -> ExpectedContext {
        self.root
    }

    fn count(&self) -> usize {
        self.stack.len() + 1
    }

    fn walk<'a>(&'a self, f: &mut ContextStackWalker<'a>) -> bool {
        let mut i = 1;
        for item in self.stack.iter().rev() {
            if !f(i, item.as_ref()) {
                return false;
            }
            i += 1;
        }
        f(i, &self.root)
    }
}

#[cfg(feature = "zc")]
unsafe impl zc::NoInteriorMut for FullContextStack {}

///////////////////////////////////////////////////////////////////////////////

#[inline(always)]
pub(crate) fn with_context<'i, F, C, T, E>(input: Input<'i>, context: C, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: WithContext<'i>,
    C: Context,
{
    match f() {
        Ok(ok) => Ok(ok),
        Err(err) => Err(err.with_context(input, context)),
    }
}
