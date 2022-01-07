//! Safely and explicitly parse untrusted aka `dangerous` data.
//!
//! See the [`guide`] module to see how to get started.

///////////////////////////////////////////////////////////////////////////////
// Library quirks & hacks
//
// # `as_any` on selected traits when used with &dyn
//
// An ideal implementation wouldn't require this function and we would just lean
// on the super trait requirement, but doesn't seem possible today with trait
// objects. See: https://github.com/rust-lang/rfcs/issues/2035

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(doc, deny(rustdoc::all))]
#![forbid(trivial_casts, trivial_numeric_casts, unstable_features)]
#![deny(
    unused,
    missing_docs,
    rust_2018_idioms,
    future_incompatible,
    clippy::all,
    clippy::correctness,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::cargo
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::type_repetition_in_bounds,
    clippy::inline_always,
    rustdoc::missing_doc_code_examples
)]
// FIXME: remove false positive, fixed in v1.59
#![cfg_attr(doc, allow(rustdoc::private_doc_tests))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod reader;
mod support;
mod util;

pub mod display;
pub mod error;
#[cfg(feature = "guide")]
pub mod guide;
pub mod input;

pub use self::error::{Error, Expected, Fatal, Invalid, ToRetryRequirement};
pub use self::input::{Bound, ByteArray, Bytes, Input, MaybeString, Span, String};
pub use self::reader::{BytesReader, Peek, Reader, StringReader};

// Re-exported types from core::fmt along with `DisplayBase` and `Write`.
// This is used crate wide with the exception of crate::display.
pub(crate) mod fmt {
    pub(crate) use crate::display::{DisplayBase, Write};
    pub(crate) use core::fmt::{Debug, Display, Error, Formatter, Result};
}

use crate::input::IntoInput;

/// Creates a new `Input` from a byte or string slice.
///
/// It is recommended to use this directly from the crate as `dangerous::input()`,
/// not as an import via `use` as shown below, as you lose the discoverability.
///
/// ```
/// use dangerous::input; // bad
///
/// dangerous::input(b"hello"); // do this instead
/// ```
#[inline(always)]
pub fn input<'i, I>(input: I) -> I::Input
where
    I: IntoInput<'i>,
{
    input.into_input()
}
