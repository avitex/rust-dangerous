#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "alloc")]
use core::iter;

use super::{Context, ExpectedContext};

/// Implemented for walkable stacks of [`Context`]s collected from an error.
pub trait Backtrace: 'static {
    /// The root context.
    fn root(&self) -> ExpectedContext;

    /// Return the total number of contexts.
    fn count(&self) -> usize;

    /// Walk the context backtrace, starting with the highest context to the root.
    ///
    /// Returns `true` if all of the stack available was walked, `false` if not.
    fn walk<'a>(&'a self, f: &mut BacktraceWalker<'a>) -> bool;
}

/// Implemented for [`Backtrace`] builders.
pub trait BacktraceBuilder {
    /// See [`WithContext::PASSTHROUGH`].
    ///
    /// [`WithContext::PASSTHROUGH`]: crate::error::WithContext::PASSTHROUGH
    const PASSTHROUGH: bool = false;

    /// Create the builder from a root expected context.
    fn from_root(context: ExpectedContext) -> Self;

    /// Push a context onto the stack.
    fn push(&mut self, context: impl Context);

    /// Push a child context attached to the last parent pushed onto the stack.
    fn push_child(&mut self, context: impl Context);
}

/// A dynamic function for walking a context backtrace.
///
/// Returns `true` if the walk should continue, `false` if not.
///
/// # Parameters
///
/// - `parent depth` (the parent depth of the context starting from `1`).
/// - `context` (the context at the provided depth).
///
/// # Parent depth
///
/// Contexts are returned from the top of the stack to the bottom. Child
/// contexts will follow after a parent context and will share the same `parent
/// depth` value.
pub type BacktraceWalker<'a> = dyn FnMut(usize, &dyn Context) -> bool + 'a;

///////////////////////////////////////////////////////////////////////////////
// Root context backtrace

/// A [`Backtrace`] that only contains the root [`ExpectedContext`].
pub struct RootBacktrace {
    context: ExpectedContext,
}

impl BacktraceBuilder for RootBacktrace {
    const PASSTHROUGH: bool = true;

    fn from_root(context: ExpectedContext) -> Self {
        Self { context }
    }

    fn push(&mut self, _context: impl Context) {}

    fn push_child(&mut self, _context: impl Context) {}
}

impl Backtrace for RootBacktrace {
    fn root(&self) -> ExpectedContext {
        self.context
    }

    fn count(&self) -> usize {
        1
    }

    fn walk<'a>(&'a self, f: &mut BacktraceWalker<'a>) -> bool {
        f(1, &self.context)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Full backtrace

/// A [`Backtrace`] that contains all [`Context`]s collected.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct FullBacktrace {
    root: ExpectedContext,
    stack: Vec<(Box<dyn Context>, bool)>,
}

#[cfg(feature = "alloc")]
impl BacktraceBuilder for FullBacktrace {
    fn from_root(context: ExpectedContext) -> Self {
        Self {
            root: context,
            stack: Vec::with_capacity(32),
        }
    }

    fn push(&mut self, context: impl Context) {
        self.stack.push((Box::new(context), true))
    }

    fn push_child(&mut self, context: impl Context) {
        self.stack.push((Box::new(context), false))
    }
}

#[cfg(feature = "alloc")]
impl Backtrace for FullBacktrace {
    fn root(&self) -> ExpectedContext {
        self.root
    }

    fn count(&self) -> usize {
        self.stack.len() + 1
    }

    fn walk<'a>(&'a self, f: &mut BacktraceWalker<'a>) -> bool {
        let root_as_dyn: &dyn Context = &self.root;
        let stack_iter = self.stack.iter().map(|(context, is_parent)| {
            let context: &dyn Context = context.as_ref();
            (context, *is_parent)
        });
        let items_iter = iter::once((root_as_dyn, true)).chain(stack_iter).rev();
        let child_iter =
            &mut items_iter
                .clone()
                .filter_map(|(context, is_parent)| if is_parent { None } else { Some(context) });
        let mut depth = 0;
        let mut children_skipped = 0;
        // Starts from the top context, with children before their parent.
        for (context, is_parent) in items_iter {
            if is_parent {
                depth += 1;
                if !f(depth, context) {
                    return false;
                }
                for child in child_iter.take(children_skipped) {
                    if !f(depth, child) {
                        return false;
                    }
                }
                children_skipped = 0;
            } else {
                children_skipped += 1;
            }
        }
        true
    }
}
