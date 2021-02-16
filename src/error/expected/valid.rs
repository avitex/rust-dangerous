use crate::error::{CoreContext, RetryRequirement, ToRetryRequirement};
use crate::fmt;
use crate::input::MaybeString;

/// An error representing a failed requirement for a valid
/// [`Input`](crate::Input).
#[must_use = "error must be handled"]
pub struct ExpectedValid<'i> {
    pub(crate) retry_requirement: Option<RetryRequirement>,
    pub(crate) context: CoreContext,
    pub(crate) input: MaybeString<'i>,
}

impl<'i> ExpectedValid<'i> {
    /// The [`CoreContext`] around the error.
    #[inline(always)]
    #[must_use]
    pub fn context(&self) -> CoreContext {
        self.context
    }

    /// The [`Input`](crate::Input) provided in the context when the error
    /// occurred.
    #[inline(always)]
    pub fn input(&self) -> MaybeString<'i> {
        self.input.clone()
    }
}

impl<'i> fmt::Debug for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("ExpectedValid");

        debug.field("retry_requirement", &self.retry_requirement);
        debug.field("context", &self.context().debug_for(self.input()));
        debug.field("input", &self.input());

        debug.finish()
    }
}

impl<'i> fmt::DisplayBase for ExpectedValid<'i> {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("expected ")?;
        self.context.expected.fmt(w)
    }
}

impl<'i> fmt::Display for ExpectedValid<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

impl<'i> ToRetryRequirement for ExpectedValid<'i> {
    #[inline]
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        if self.is_fatal() {
            None
        } else {
            self.retry_requirement
        }
    }

    #[inline]
    fn is_fatal(&self) -> bool {
        self.input.is_bound() || self.retry_requirement.is_none()
    }
}
