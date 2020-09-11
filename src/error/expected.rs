use core::fmt;

use crate::error::{
    Context, Error, ErrorDetails, ErrorDisplay, RetryRequirement, ToRetryRequirement,
};
use crate::input::Input;
use crate::utils::ByteCount;

#[cfg(any(feature = "std", feature = "alloc"))]
use self::context_node::ContextNode;

/// A catch-all error for all expected errors supported in this crate.
#[derive(Debug)]
pub struct Expected<'i> {
    inner: ExpectedInner<'i>,
    #[cfg(any(feature = "std", feature = "alloc"))]
    context: ContextNode,
}

#[derive(Debug)]
enum ExpectedInner<'i> {
    /// An exact value was expected in a context.
    Value(ExpectedValue<'i>),
    /// A valid value was expected in a context.
    Valid(ExpectedValid<'i>),
    /// A length was expected in a context.
    Length(ExpectedLength<'i>),
}

impl<'i> Expected<'i> {
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    fn details(&self) -> &(dyn ErrorDetails<'i>) {
        match self.inner {
            ExpectedInner::Value(ref err) => err,
            ExpectedInner::Valid(ref err) => err,
            ExpectedInner::Length(ref err) => err,
        }
    }

    fn update_input(&mut self, input: &'i Input) {
        match self.inner {
            ExpectedInner::Value(ref mut err) => err.update_input(input),
            ExpectedInner::Valid(ref mut err) => err.update_input(input),
            ExpectedInner::Length(ref mut err) => err.update_input(input),
        }
    }
}

impl<'i> ErrorDetails<'i> for Expected<'i> {
    fn input(&self) -> &'i Input {
        self.details().input()
    }

    fn span(&self) -> &'i Input {
        self.details().span()
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn context(&self) -> &dyn Context {
        &self.context
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn context(&self) -> &dyn Context {
        self.details().context()
    }

    fn found_value(&self) -> Option<&Input> {
        self.details().found_value()
    }

    fn expected_value(&self) -> Option<&Input> {
        self.details().expected_value()
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.details().description(f)
    }
}

impl<'i> ToRetryRequirement for Expected<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        match self.inner {
            ExpectedInner::Value(ref err) => err.to_retry_requirement(),
            ExpectedInner::Valid(ref err) => err.to_retry_requirement(),
            ExpectedInner::Length(ref err) => err.to_retry_requirement(),
        }
    }
}

impl<'i> Error<'i> for Expected<'i> {
    fn with_context<C>(mut self, input: &'i Input, context: C) -> Self
    where
        C: Context,
    {
        let _ = &context;
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.context = self.context.with_parent(context);
        }
        self.update_input(input);
        self
    }
}

impl<'i> From<ExpectedLength<'i>> for Expected<'i> {
    fn from(err: ExpectedLength<'i>) -> Self {
        Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            context: ContextNode::new(err.context().operation()),
            inner: ExpectedInner::Length(err),
        }
    }
}

impl<'i> From<ExpectedValid<'i>> for Expected<'i> {
    fn from(err: ExpectedValid<'i>) -> Self {
        Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            context: ContextNode::new(err.context().operation()),
            inner: ExpectedInner::Valid(err),
        }
    }
}

impl<'i> From<ExpectedValue<'i>> for Expected<'i> {
    fn from(err: ExpectedValue<'i>) -> Self {
        Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            context: ContextNode::new(err.context().operation()),
            inner: ExpectedInner::Value(err),
        }
    }
}

impl_error_common!(Expected);

///////////////////////////////////////////////////////////////////////////////
// Expected value error

/// An error representing a failed exact value requirement of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValue<'i> {
    pub(crate) value: &'i Input,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) operation: &'static str,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] value that was expected.
    pub fn expected(&self) -> &Input {
        self.value
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValue<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn expected_value(&self) -> Option<&Input> {
        Some(self.value)
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("found a different value to the exact expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        let needed = self.value.len();
        let had = self.span().len();
        RetryRequirement::from_had_and_needed(had, needed)
    }
}

impl<'i> Error<'i> for ExpectedValue<'i> {
    fn with_context<C>(mut self, input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self.update_input(input);
        self
    }
}

impl_error_common!(ExpectedValue);

///////////////////////////////////////////////////////////////////////////////
// Expected length error

/// An error representing a failed requirement for a length of [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedLength<'i> {
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) operation: &'static str,
}

impl<'i> ExpectedLength<'i> {
    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use [`ErrorDetails::retry_requirement()`].
    pub fn min(&self) -> usize {
        self.min
    }

    /// The maximum length that was expected in a context, if applicable.
    ///
    /// If max has a value, this signifies the [`Input`] exceeded it in some
    /// way. An example of this would be [`Input::read_all`], where there was
    /// [`Input`] left over.
    pub fn max(&self) -> Option<usize> {
        self.max
    }

    /// Returns `true` if an exact length was expected in a context.
    pub fn is_exact(&self) -> bool {
        Some(self.min) == self.max
    }

    /// Returns `true` if `max()` has a value.
    pub fn is_fatal(&self) -> bool {
        self.max.is_some()
    }

    /// The exact length that was expected in a context, if applicable.
    ///
    /// Will return a value if `is_exact()` returns `true`.
    pub fn exact(&self) -> Option<usize> {
        if self.is_exact() {
            self.max
        } else {
            None
        }
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedLength<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn expected_value(&self) -> Option<&Input> {
        None
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "found {} when ", ByteCount(self.span().len()))?;
        match (self.min, self.max) {
            (0, Some(max)) => write!(f, "at most {}", ByteCount(max)),
            (min, None) => write!(f, "at least {}", ByteCount(min)),
            (min, Some(max)) if min == max => write!(f, "exactly {}", ByteCount(min)),
            (min, Some(max)) => write!(
                f,
                "at least {} and at most {}",
                ByteCount(min),
                ByteCount(max)
            ),
        }?;
        write!(f, " was expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedLength<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let had = self.span().len();
            let needed = self.min;
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }
}

impl<'i> Error<'i> for ExpectedLength<'i> {
    fn with_context<C>(mut self, input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self.update_input(input);
        self
    }
}

impl_error_common!(ExpectedLength);

///////////////////////////////////////////////////////////////////////////////
// Expected valid error

/// An error representing a failed requirement for a valid [`Input`].
#[derive(Debug, Clone)]
pub struct ExpectedValid<'i> {
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) operation: &'static str,
    pub(crate) expected: &'static str,
    pub(crate) retry_requirement: Option<RetryRequirement>,
}

impl<'i> ExpectedValid<'i> {
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }

    fn update_input(&mut self, input: &'i Input) {
        if self.input.is_within(input) {
            self.input = input;
        }
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValid<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn context(&self) -> &dyn Context {
        &self.operation
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn expected_value(&self) -> Option<&Input> {
        None
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {}", self.expected)
    }
}

impl<'i> ToRetryRequirement for ExpectedValid<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

impl<'i> Error<'i> for ExpectedValid<'i> {
    fn with_context<C>(mut self, input: &'i Input, _context: C) -> Self
    where
        C: Context,
    {
        self.update_input(input);
        self
    }
}

impl_error_common!(ExpectedValid);

#[cfg(any(feature = "std", feature = "alloc"))]
mod context_node {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;

    #[derive(Debug)]
    pub(super) struct ContextNode {
        this: Box<dyn Context>,
        child: Option<Box<dyn Context>>,
    }

    impl ContextNode {
        pub(super) fn new<C>(context: C) -> Self
        where
            C: Context,
        {
            Self {
                this: Box::new(context),
                child: None,
            }
        }

        pub(super) fn with_parent<C>(self, parent: C) -> Self
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
}
