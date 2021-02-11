use core::any::Any;

use crate::fmt;
use crate::input::{Input, MaybeString, Span};

use super::WithContext;

/// Information surrounding an error.
pub trait Context: 'static {
    fn span(&self) -> Option<Span> {
        None
    }

    /// Returns the operation that failed in this context.
    fn operation(&self) -> &dyn Operation;

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

    fn is_child(&self) -> bool {
        false
    }
}

/// Operation that failed within a context.
pub trait Operation: Any {
    /// Description of the operation in a simple manner, for informing a user
    /// what is trying to be achieved.
    ///
    /// The description should make sense in the following sentence if you were
    /// to substitute it:
    ///
    /// ```text
    /// error attempting to <operation>.
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`fmt::Error`] if failed to write to the formatter.
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result;

    /// Return a reference of self as [`Any`].
    fn as_any(&self) -> &dyn Any;
}

///////////////////////////////////////////////////////////////////////////////
// Basic expected context

impl Context for &'static str {
    fn operation(&self) -> &dyn Operation {
        &CoreOperation::Context
    }

    fn has_expected(&self) -> bool {
        true
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self)
    }
}

impl Operation for &'static str {
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// External context

/// A [`Context`] for external operations.
#[derive(Copy, Clone)]
pub struct ExternalContext<O, E> {
    /// Value for [`Context::operation()`].
    pub operation: Option<O>,
    /// Value for [`Context::expected()`].
    pub expected: Option<E>,
}

impl<O, E> Context for ExternalContext<O, E>
where
    O: Operation,
    E: fmt::DisplayBase + 'static,
{
    fn operation(&self) -> &dyn Operation {
        match &self.operation {
            None => &CoreOperation::Context,
            Some(operation) => operation,
        }
    }

    fn has_expected(&self) -> bool {
        self.expected.is_some()
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match &self.expected {
            None => Err(fmt::Error),
            Some(expected) => expected.fmt(w),
        }
    }
}

impl<O, E> fmt::Debug for ExternalContext<O, E>
where
    O: fmt::Debug,
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExternalContext")
            .field("operation", &self.operation)
            .field("expected", &self.expected)
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Core context

/// A [`Context`] for core operations supported by `dangerous`.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct CoreContext {
    pub span: Span,
    /// Value for [`Context::operation()`].
    pub operation: CoreOperation,
    /// Value for [`Context::expected()`].
    pub expected: CoreExpected,
}

impl CoreContext {
    pub fn debug_for(self, input: MaybeString<'_>) -> DebugFor<'_> {
        DebugFor {
            input,
            context: self,
        }
    }

    pub(crate) fn from_operation(operation: CoreOperation, span: Span) -> Self {
        Self {
            span,
            operation,
            expected: CoreExpected::Unknown,
        }
    }
}

impl Context for CoreContext {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }

    fn operation(&self) -> &dyn Operation {
        &self.operation
    }

    fn has_expected(&self) -> bool {
        self.expected != CoreExpected::Unknown
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        fmt::DisplayBase::fmt(&self.expected, w)
    }
}

impl fmt::Debug for CoreContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreContext")
            .field("span", &self.span)
            .field("operation", &self.operation)
            .field("expected", &self.expected)
            .finish()
    }
}

pub struct DebugFor<'i> {
    input: MaybeString<'i>,
    context: CoreContext,
}

impl<'i> fmt::Debug for DebugFor<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreContext")
            .field("span", &self.context.span.debug_for(self.input.clone()))
            .field("operation", &self.context.operation)
            .field("expected", &self.context.expected)
            .finish()
    }
}

/// Core operations used by `dangerous`.
#[allow(missing_docs)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CoreOperation {
    // Context
    Context,
    // Entry
    ReadAll,
    ReadPartial,
    // Consuming
    Consume,
    // Skipping
    Skip,
    SkipWhile,
    SkipStrWhile,
    SkipUntil,
    SkipUntilConsume,
    // Taking
    Take,
    TakeUntil,
    TakeUntilConsume,
    TakeWhile,
    TakeConsumed,
    TakeStrWhile,
    TakeRemainingStr,
    // Peeking
    Peek,
    PeekU8,
    PeekChar,
    // Reading
    ReadU8,
    ReadI8,
    ReadU16LE,
    ReadI16LE,
    ReadU16BE,
    ReadI16BE,
    ReadU32LE,
    ReadI32LE,
    ReadU32BE,
    ReadI32BE,
    ReadU64LE,
    ReadI64LE,
    ReadU64BE,
    ReadI64BE,
    ReadU128LE,
    ReadI128LE,
    ReadU128BE,
    ReadI128BE,
    ReadF32LE,
    ReadF32BE,
    ReadF64LE,
    ReadF64BE,
    ReadChar,
    // Errors
    RecoverIf,
    Verify,
    Expect,
    ExpectExternal,
    // Converting
    IntoNonEmpty,
    IntoExternal,
    IntoString,
}

impl Operation for CoreOperation {
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(CoreOperation::description(*self))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CoreOperation {
    fn description(self) -> &'static str {
        match self {
            Self::Context => "<context>",
            Self::ReadAll => "read all input",
            Self::ReadPartial => "read a partial length of input",
            Self::Consume => "consume input",
            Self::Skip => "skip a length of input",
            Self::SkipWhile => "skip input while a pattern matches",
            Self::SkipUntil => "skip input until a pattern matches",
            Self::SkipUntilConsume => "skip input until a pattern matches and consume it",
            Self::SkipStrWhile => "skip UTF-8 input while a condition remains true",
            Self::Take => "take a length of input",
            Self::TakeWhile => "take input while a pattern matches",
            Self::TakeUntil => "take input until a pattern matches",
            Self::TakeUntilConsume => "take input until a pattern matches and consume it",
            Self::TakeConsumed => "take input that was consumed",
            Self::TakeStrWhile => "take UTF-8 input while a condition remains true",
            Self::TakeRemainingStr => "take remaining string within bytes",
            Self::Peek => "peek a length of input",
            Self::PeekU8 => "peek a u8",
            Self::PeekChar => "peek a char",
            Self::ReadU8 => "read a u8",
            Self::ReadI8 => "read a i8",
            Self::ReadU16LE => "read a little-endian encoded u16",
            Self::ReadI16LE => "read a little-endian encoded i16",
            Self::ReadU16BE => "read a big-endian encoded u16",
            Self::ReadI16BE => "read a big-endian encoded i16",
            Self::ReadU32LE => "read a little-endian encoded u32",
            Self::ReadI32LE => "read a little-endian encoded i32",
            Self::ReadU32BE => "read a big-endian encoded u32",
            Self::ReadI32BE => "read a big-endian encoded i32",
            Self::ReadU64LE => "read a little-endian encoded u64",
            Self::ReadI64LE => "read a little-endian encoded i64",
            Self::ReadU64BE => "read a big-endian encoded u64",
            Self::ReadI64BE => "read a big-endian encoded i64",
            Self::ReadU128LE => "read a little-endian encoded u128",
            Self::ReadI128LE => "read a little-endian encoded i128",
            Self::ReadU128BE => "read a big-endian encoded u128",
            Self::ReadI128BE => "read a big-endian encoded i128",
            Self::ReadF32LE => "read a little-endian encoded f32",
            Self::ReadF32BE => "read a big-endian encoded f32",
            Self::ReadF64LE => "read a little-endian encoded f64",
            Self::ReadF64BE => "read a big-endian encoded f64",
            Self::ReadChar => "read a char",
            Self::RecoverIf => "recover if a condition returns true",
            Self::Verify => "read and verify input",
            Self::Expect => "read and expect a value",
            Self::ExpectExternal => "read and expect an external value",
            Self::IntoNonEmpty => "convert input into non-empty input",
            Self::IntoExternal => "convert input into external type",
            Self::IntoString => "convert input into string",
        }
    }
}

impl fmt::DisplayBase for CoreOperation {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.description(w)
    }
}

/// Core expectations used by `dangerous`.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CoreExpected {
    /// What is expected is unknown.
    ///
    /// This is used to return `false` on [`Context::has_expected()`].
    Unknown,
    /// Non empty input was expected.
    NonEmpty,
    /// An exact value was expected.
    ExactValue,
    /// A pattern match was expected.
    PatternMatch,
    /// No trailing input was expected.
    NoTrailingInput,
    /// Contains the description of the value that was expected.
    Valid(&'static str),
    /// Enough input for a given description of a value was expected.
    EnoughInputFor(&'static str),
}

impl fmt::DisplayBase for CoreExpected {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match *self {
            Self::Unknown => w.write_str("unknown"),
            Self::NonEmpty => w.write_str("non-empty input"),
            Self::ExactValue => w.write_str("exact value"),
            Self::PatternMatch => w.write_str("pattern match"),
            Self::NoTrailingInput => w.write_str("no trailing input"),
            Self::Valid(expected) => w.write_str(expected),
            Self::EnoughInputFor(expected) => {
                w.write_str("enough input for ")?;
                w.write_str(expected)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

// FIXME
#[inline(always)]
pub(crate) fn with_context<'i, F, T, E>(
    context: impl Context,
    input: impl Input<'i>,
    f: F,
) -> Result<T, E>
where
    E: WithContext<'i>,
    F: FnOnce() -> Result<T, E>,
{
    match f() {
        Ok(ok) => Ok(ok),
        Err(err) => Err(err.with_context(context).with_input(input)),
    }
}

pub struct WithChildContext<E>(E);

impl<E> WithChildContext<E> {
    pub fn new(inner: E) -> Self {
        Self(inner)
    }

    pub fn unwrap(self) -> E {
        self.0
    }
}

impl<'i, E> WithContext<'i> for WithChildContext<E>
where
    E: WithContext<'i>,
{
    const PASSTHROUGH: bool = E::PASSTHROUGH;

    fn with_input(self, _input: impl Input<'i>) -> Self {
        self
    }

    fn with_context(self, context: impl Context) -> Self {
        Self(self.0.with_context(ChildContext(context)))
    }
}

struct ChildContext<T>(T);

impl<T> Context for ChildContext<T>
where
    T: Context,
{
    fn operation(&self) -> &dyn Operation {
        self.0.operation()
    }

    fn has_expected(&self) -> bool {
        self.0.has_expected()
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        self.0.expected(w)
    }

    fn is_child(&self) -> bool {
        true
    }
}
