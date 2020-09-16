use core::any::Any;
use core::fmt::{self, Debug};

/// The base context surrounding an error.
pub trait Context: Any + Debug {
    /// The operation that was attempted when an error occured.
    ///
    /// It should described in a simple manner what is trying to be achieved and
    /// make sense in the following sentence if you were to substitute it:
    ///
    /// ```text
    /// Something failed while attempting to <operation> from the input.
    /// ```
    fn operation(&self) -> &'static str;

    /// Returns a [`fmt::Display`] formattable value of what was expected.
    fn expected(&self) -> Option<&dyn fmt::Display>;
}

pub trait ContextStack {
    fn push<C>(&mut self, context: C)
    where
        C: Context;

    fn walk<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnMut(usize, &dyn Context) -> Result<(), E>;
}

#[cfg(feature = "full-context")]
use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "full-context")]
#[derive(Default)]
pub struct FullContextStack {
    root: Option<RootContext>,
    stack: Vec<Box<dyn Context>>,
}

#[cfg(feature = "full-context")]
impl ContextStack for FullContextStack {
    fn push<C>(&mut self, context: C)
    where
        C: Context,
    {
        if let Some(root) = Any::downcast_ref::<RootContext>(&context) {
            self.root = Some(*root);
        } else {
            self.stack.push(Box::new(context))
        }
    }

    fn walk<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnMut(usize, &dyn Context) -> Result<(), E>,
    {
        if let Some(root) = self.root {
            f(1, &root)
        } else {
            Ok(())
        }
    }
}

pub struct ContextDisplay<'a, T> {
    stack: &'a T,
}

impl<'a, T> ContextDisplay<'a, T> {
    pub fn new(stack: &'a T) -> Self {
        Self { stack }
    }
}

impl<'a, T> fmt::Display for ContextDisplay<'a, T>
where
    T: ContextStack,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.stack.walk(|i, c| {
            write!(f, "\n  {}. `{}`", i, c.operation())?;
            if let Some(expected) = c.expected() {
                write!(f, " (expected {})", expected)?;
            }
            Ok(())
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Basic expected context

impl Context for &'static str {
    fn operation(&self) -> &'static str {
        "read"
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        Some(self)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Operation context

#[derive(Clone, Copy, Debug)]
pub(crate) struct OperationContext(pub(crate) &'static str);

impl Context for OperationContext {
    fn operation(&self) -> &'static str {
        self.0
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        None
    }
}

///////////////////////////////////////////////////////////////////////////////
// Root context

#[derive(Clone, Copy, Debug)]
pub(crate) struct RootContext {
    pub(crate) operation: &'static str,
    pub(crate) expected: &'static str,
}

impl Context for RootContext {
    fn operation(&self) -> &'static str {
        self.operation
    }

    fn expected(&self) -> Option<&dyn fmt::Display> {
        Some(&self.expected)
    }
}

impl ContextStack for RootContext {
    fn push<C>(&mut self, context: C)
    where
        C: Context,
    {
    }

    fn walk<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnMut(usize, &dyn Context) -> Result<(), E>,
    {
        f(1, self)
    }
}
