//! Error support.
//!
//! - If you want the fastest error which has no debugging information,
//!   [`Fatal`] or [`Invalid`] (retryable) has you covered.
//! - If you want an error that is still designed to be fast, but also includes
//!   debugging information, [`Expected`] will meet your uh, expectations... If
//!   the feature `full-context` is enabled, [`Expected`] uses
//!   [`FullContextStack`], [`RootContextStack`] if not.
//! - If you require more verbosity, consider creating custom [`Context`]s
//!   before jumping to custom errors. If you do require a custom error,
//!   implementing it is easy enough. Just implement [`FromContext`] and
//!   [`From`] for [`ExpectedValue`], [`ExpectedLength`] and [`ExpectedValid`]
//!   and you'll be on your merry way. Additionally implement [`Details`] to
//!   support lovely error printing and [`ToRetryRequirement`] for streaming
//!   protocols.
//!
//! Most of what `dangerous` supports out of the box is good to go. If you need
//! to stretch out performance more, or provide additional functionality on what
//! is provided, the error system should be flexible for those requirements. If
//! it's not, consider opening an issue.

mod context;
mod expected;
mod fatal;
mod invalid;
mod retry;
mod traits;

#[cfg(feature = "full-context")]
pub use self::context::FullContextStack;
pub use self::context::{
    Context, ContextStack, ContextStackBuilder, ContextStackWalker, ExpectedContext,
    RootContextStack,
};
pub use self::expected::{Expected, ExpectedLength, ExpectedValid, ExpectedValue};
pub use self::fatal::Fatal;
pub use self::invalid::Invalid;
pub use self::retry::{RetryRequirement, ToRetryRequirement};
pub use self::traits::{Details, Error, FromContext, FromExpected};

pub(crate) use self::context::{with_context, OperationContext};
