use core::fmt;

use crate::error::{
    fmt_debug_error, Context, ContextStack, Error, ErrorDetails, ErrorDisplay, Invalid,
    RetryRequirement, RootContext, RootContextStack, ToRetryRequirement,
};
use crate::input::{input, Input};
use crate::utils::ByteCount;

#[cfg(feature = "full-context")]
type ExpectedStack = crate::error::FullContextStack;
#[cfg(not(feature = "full-context"))]
type ExpectedStack = crate::error::EmptyContextStack;

// /// A catch-all error for all expected errors supported in this crate.
// pub struct Expected<'i, S = ExpectedStack>
// where
//     S: ContextStack,
// {
//     stack: S,
//     input: &'i Input,
//     inner: ExpectedInner<'i>,
// }

// enum ExpectedInner<'i> {
//     /// An exact value was expected in a context.
//     Value(ExpectedValue<'i>),
//     /// A valid value was expected in a context.
//     Valid(ExpectedValid<'i>),
//     /// A length was expected in a context.
//     Length(ExpectedLength<'i>),
// }

// impl<'i, S> Expected<'i, S>
// where
//     S: ContextStack + Default,
// {
//     /// Returns an `ErrorDisplay` for formatting.
//     pub fn display(&self) -> ErrorDisplay<&Self> {
//         ErrorDisplay::new(self)
//     }

//     fn from_inner(inner: ExpectedInner<'i>) -> Self {
//         Self {
//             inner,
//             stack: Default::default(),
//         }
//     }
// }

// impl<'i, S> ErrorDetails<'i, S> for Expected<'i, S>
// where
//     S: ContextStack,
// {
//     fn input(&self) -> &'i Input {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.input(),
//             ExpectedInner::Valid(ref err) => err.input(),
//             ExpectedInner::Length(ref err) => err.input(),
//         }
//     }

//     fn span(&self) -> &'i Input {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.span(),
//             ExpectedInner::Valid(ref err) => err.span(),
//             ExpectedInner::Length(ref err) => err.input(),
//         }
//     }

//     fn found_value(&self) -> Option<&Input> {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.found_value(),
//             ExpectedInner::Valid(ref err) => err.found_value(),
//             ExpectedInner::Length(ref err) => err.found_value(),
//         }
//     }

//     fn expected_value(&self) -> Option<&Input> {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.expected_value(),
//             ExpectedInner::Valid(ref err) => err.expected_value(),
//             ExpectedInner::Length(ref err) => err.expected_value(),
//         }
//     }

//     fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.description(f),
//             ExpectedInner::Valid(ref err) => err.description(f),
//             ExpectedInner::Length(ref err) => err.description(f),
//         }
//     }
// }

// impl<'i, S> ToRetryRequirement for Expected<'i, S>
// where
//     S: ContextStack,
// {
//     fn to_retry_requirement(&self) -> Option<RetryRequirement> {
//         match self.inner {
//             ExpectedInner::Value(ref err) => err.to_retry_requirement(),
//             ExpectedInner::Valid(ref err) => err.to_retry_requirement(),
//             ExpectedInner::Length(ref err) => err.to_retry_requirement(),
//         }
//     }
// }

// impl<'i, S> Error<'i> for Expected<'i, S>
// where
//     S: ContextStack,
// {
//     fn from_input_context<C>(mut self, input: &'i Input, context: C) -> Self
//     where
//         C: Context,
//     {
//         let curr_input = match self.inner {
//             ExpectedInner::Value(ref mut err) => &mut err.input,
//             ExpectedInner::Valid(ref mut err) => &mut err.input,
//             ExpectedInner::Length(ref mut err) => &mut err.input,
//         };
//         if curr_input.is_within(input) {
//             *curr_input = input
//         }
//         self.stack.push(context);
//         self
//     }
// }

// impl<'i> fmt::Display for Expected<'i> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.display().fmt(f)
//     }
// }

// impl<'i> fmt::Debug for Expected<'i> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         fmt_debug_error(self, f)
//     }
// }

// impl<'i, S> From<ExpectedLength<'i>> for Expected<'i, S>
// where
//     S: ContextStack + Default,
// {
//     fn from(err: ExpectedLength<'i>) -> Self {
//         Self::from_inner(ExpectedInner::Length(err))
//     }
// }

// impl<'i, S> From<ExpectedValid<'i>> for Expected<'i, S>
// where
//     S: ContextStack + Default,
// {
//     fn from(err: ExpectedValid<'i>) -> Self {
//         Self::from_inner(ExpectedInner::Valid(err))
//     }
// }

// impl<'i, S> From<ExpectedValue<'i>> for Expected<'i, S>
// where
//     S: ContextStack + Default,
// {
//     fn from(err: ExpectedValue<'i>) -> Self {
//         Self::from_inner(ExpectedInner::Value(err))
//     }
// }

// impl<'i> From<Expected<'i>> for Invalid {
//     fn from(err: Expected<'i>) -> Self {
//         err.to_retry_requirement().into()
//     }
// }

// #[cfg(feature = "std")]
// impl<'i> std::error::Error for Expected<'i> {}

///////////////////////////////////////////////////////////////////////////////
// Expected value error

#[derive(Clone)]
#[allow(variant_size_differences)]
pub(crate) enum Value<'a> {
    Byte(u8),
    Bytes(&'a [u8]),
}

impl<'i> Value<'i> {
    pub(crate) fn as_input(&self) -> &Input {
        match self {
            Self::Byte(ref b) => Input::from_u8(b),
            Self::Bytes(bytes) => input(bytes),
        }
    }
}

/// An error representing a failed exact value requirement of [`Input`].
#[derive(Clone)]
pub struct ExpectedValue<'i> {
    pub(crate) value: Value<'i>,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) stack: RootContextStack,
}

impl<'i> ExpectedValue<'i> {
    /// The [`Input`] value that was expected.
    pub fn expected(&self) -> &Input {
        self.value.as_input()
    }

    /// Returns `true` if the value could never match and `true` if the matching
    /// was incomplete.
    pub fn is_fatal(&self) -> bool {
        !self.value.as_input().has_prefix(self.span.as_dangerous())
    }

    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValue<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn expected_value(&self) -> Option<&Input> {
        Some(self.value.as_input())
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("found a different value to the exact expected")
    }
}

impl<'i> ToRetryRequirement for ExpectedValue<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            let needed = self.value.as_input().len();
            let had = self.span().len();
            RetryRequirement::from_had_and_needed(had, needed)
        }
    }
}

impl<'i> fmt::Display for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'i> fmt::Debug for ExpectedValue<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_debug_error(self, f)
    }
}

impl<'i> From<ExpectedValue<'i>> for Invalid {
    fn from(err: ExpectedValue<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

#[cfg(feature = "std")]
impl<'i> std::error::Error for ExpectedValue<'i> {}

///////////////////////////////////////////////////////////////////////////////
// Expected length error

/// An error representing a failed requirement for a length of [`Input`].
#[derive(Clone)]
pub struct ExpectedLength<'i> {
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) stack: RootContext,
}

impl<'i> ExpectedLength<'i> {
    /// The minimum length that was expected in a context.
    ///
    /// This doesn't not take into account the section of input being processed
    /// when this error occurred. If you wish to work out the requirement to
    /// continue processing input use
    /// [`ToRetryRequirement::to_retry_requirement()`].
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
}

impl<'i> ErrorDetails<'i, RootContextStack> for ExpectedLength<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
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

impl<'i> fmt::Display for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'i> fmt::Debug for ExpectedLength<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_debug_error(self, f)
    }
}

impl<'i> From<ExpectedLength<'i>> for Invalid {
    fn from(err: ExpectedLength<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

#[cfg(feature = "std")]
impl<'i> std::error::Error for ExpectedLength<'i> {}

///////////////////////////////////////////////////////////////////////////////
// Expected valid error

/// An error representing a failed requirement for a valid [`Input`].
#[derive(Clone)]
pub struct ExpectedValid<'i> {
    pub(crate) span: &'i Input,
    pub(crate) input: &'i Input,
    pub(crate) context: RootContext,
    pub(crate) retry_requirement: Option<RetryRequirement>,
}

impl<'i> ExpectedValid<'i> {
    /// Returns an `ErrorDisplay` for formatting.
    pub fn display(&self) -> ErrorDisplay<&Self> {
        ErrorDisplay::new(self)
    }
}

impl<'i> ErrorDetails<'i> for ExpectedValid<'i> {
    fn input(&self) -> &'i Input {
        self.input
    }

    fn span(&self) -> &'i Input {
        self.span
    }

    fn found_value(&self) -> Option<&Input> {
        Some(self.input)
    }

    fn expected_value(&self) -> Option<&Input> {
        None
    }

    fn description(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}", self.context.expected)
    }
}

impl<'i> ToRetryRequirement for ExpectedValid<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        self.retry_requirement
    }
}

impl<'i> fmt::Display for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'i> fmt::Debug for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_debug_error(self, f)
    }
}

impl<'i> From<ExpectedValid<'i>> for Invalid {
    fn from(err: ExpectedValid<'i>) -> Self {
        err.to_retry_requirement().into()
    }
}

#[cfg(feature = "std")]
impl<'i> std::error::Error for ExpectedValid<'i> {}

///////////////////////////////////////////////////////////////////////////////

/// Convenience trait for specifying a catch of all possible expected errors.
pub trait FromExpected<'i>:
    Error<'i> + From<ExpectedLength<'i>> + From<ExpectedValue<'i>> + From<ExpectedValid<'i>>
{
}

impl<'i, T> FromExpected<'i> for T where
    T: Error<'i> + From<ExpectedLength<'i>> + From<ExpectedValue<'i>> + From<ExpectedValid<'i>>
{
}
