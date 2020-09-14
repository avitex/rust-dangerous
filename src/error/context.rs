use core::any::Any;
use core::fmt::Debug;

/// The context surrounding an error.
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

    /// The value expected in this context.
    fn expected(&self) -> Option<&'static str> {
        None
    }

    /// The more granular context of where the error occured.
    ///
    /// # Example
    ///
    /// Say we attempted to process a UTF-8 string from the input via
    /// [`Input::to_dangerous_str()`] within a parent operation described
    /// `decode name`. The final context produced would be that of around
    /// `decode name`. The `child` context would be that of
    /// [`Input::to_dangerous_str()`].
    ///
    /// This would allow us to walk the contexts, so we can present the
    /// following information for use in debugging:
    ///
    /// ```text
    /// UTF-8 error occured while attempting to decode name from the input.
    ///
    /// context backtrace:
    /// 1. `decode name` (expected valid name)
    /// 2. `decode utf-8 code point` (expected valid utf-8 code point)
    /// ```
    ///
    /// [`Input::to_dangerous_str()`]: crate::Input::to_dangerous_str()
    fn child(&self) -> Option<&dyn Context> {
        None
    }

    /// The number of child contexts consolidated into `self`.
    ///
    /// Any context returned from `child` is the next deeper than those that
    /// were consolidated.
    fn consolidated(&self) -> usize {
        0
    }
}

impl Context for &'static str {
    fn operation(&self) -> &'static str {
        "read"
    }

    fn expected(&self) -> Option<&'static str> {
        Some(self)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub(crate) struct OperationContext(pub(crate) &'static str);

impl Context for OperationContext {
    fn operation(&self) -> &'static str {
        self.0
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExpectedContext {
    pub(crate) operation: &'static str,
    pub(crate) expected: &'static str,
}

impl Context for ExpectedContext {
    fn operation(&self) -> &'static str {
        self.operation
    }

    fn expected(&self) -> Option<&'static str> {
        Some(self.expected)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) use self::context_node::ContextNode;

#[cfg(any(feature = "std", feature = "alloc"))]
mod context_node {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;

    #[derive(Debug)]
    struct Inner {
        operation: &'static str,
        expected: Option<&'static str>,
    }

    impl Context for Inner {
        fn operation(&self) -> &'static str {
            self.operation
        }

        fn expected(&self) -> Option<&'static str> {
            self.expected
        }
    }

    #[derive(Debug)]
    pub(crate) struct ContextNode {
        this: Box<dyn Context>,
        child: Option<Box<dyn Context>>,
    }

    impl ContextNode {
        pub(crate) fn new(context: &dyn Context) -> Self {
            Self {
                this: Box::new(Inner {
                    operation: context.operation(),
                    expected: context.expected(),
                }),
                child: None,
            }
        }

        pub(crate) fn with_parent<C>(self, parent: C) -> Self
        where
            C: Context,
        {
            Self {
                this: Box::new(parent),
                child: Some(Box::new(self)),
            }
        }
    }

    impl Context for ContextNode {
        fn child(&self) -> Option<&dyn Context> {
            self.child.as_ref().map(AsRef::as_ref)
        }

        fn consolidated(&self) -> usize {
            0
        }

        fn expected(&self) -> Option<&'static str> {
            self.this.expected()
        }

        fn operation(&self) -> &'static str {
            self.this.operation()
        }
    }
}
