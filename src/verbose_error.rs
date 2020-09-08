use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::error::{
    Context, Error, ErrorDetails, Expected, ExpectedLength, ExpectedValid, ExpectedValue,
    RetryRequirement,
};
use crate::error_display::ErrorDisplay;
use crate::input::Input;

pub struct VerboseError<'i> {
    expected: Expected<'i>,
    context: ContextNode,
}

impl<'i> VerboseError<'i> {
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }
}

impl<'i> Error<'i> for VerboseError<'i> {
    fn with_context<C>(mut self, input: &'i Input, context: C) -> Self
    where
        C: Context,
    {
        let operation = context.operation();
        self.context = self.context.with_parent(context);
        self.expected = self.expected.with_context(input, operation);
        self
    }
}

impl<'i> From<ExpectedLength<'i>> for VerboseError<'i> {
    fn from(err: ExpectedLength<'i>) -> Self {
        Self {
            context: ContextNode::new(err.context().operation()),
            expected: err.into(),
        }
    }
}

impl<'i> From<ExpectedValid<'i>> for VerboseError<'i> {
    fn from(err: ExpectedValid<'i>) -> Self {
        Self {
            context: ContextNode::new(err.context().operation()),
            expected: err.into(),
        }
    }
}

impl<'i> From<ExpectedValue<'i>> for VerboseError<'i> {
    fn from(err: ExpectedValue<'i>) -> Self {
        Self {
            context: ContextNode::new(err.context().operation()),
            expected: err.into(),
        }
    }
}

impl<'i> ErrorDetails<'i> for VerboseError<'i> {
    fn input(&self) -> &'i Input {
        self.expected.input()
    }

    fn span(&self) -> &'i Input {
        self.expected.span()
    }

    fn context(&self) -> &dyn Context {
        &self.context
    }

    fn found_value(&self) -> Option<&Input> {
        self.expected.found_value()
    }

    fn found_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expected.found_description(f)
    }

    fn expected_value(&self) -> Option<&Input> {
        self.expected.expected_value()
    }

    fn expected_description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expected.expected_description(f)
    }

    fn retry_requirement(&self) -> Option<RetryRequirement> {
        self.expected.retry_requirement()
    }
}

struct ContextNode {
    this: Box<dyn Context>,
    child: Option<Box<dyn Context>>,
}

impl ContextNode {
    fn new<C>(context: C) -> Self
    where
        C: Context,
    {
        Self {
            this: Box::new(context),
            child: None,
        }
    }

    fn with_parent<C>(self, parent: C) -> Self
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

    fn operation(&self) -> &'static str {
        self.this.operation()
    }
}
