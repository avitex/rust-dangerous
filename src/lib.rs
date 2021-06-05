//! Safely and explicitly parse untrusted aka `dangerous` data.
//!
//! See the [`guide`] module to see how to get started.
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    future_incompatible
)]
#![deny(
    unused,
    missing_docs,
    rust_2018_idioms,
    rustdoc::all,
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
    clippy::inline_always
)]
// FIXME: remove false positives
#![allow(
    // https://github.com/rust-lang/rust-clippy/issues/5822
    clippy::option_if_let_else,
    // https://github.com/rust-lang/rust/issues/72081
    private_doc_tests,
)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod input;
mod reader;
mod support;
mod util;

pub mod display;
pub mod error;
#[cfg(feature = "guide")]
pub mod guide;

pub use self::error::{Error, Expected, Fatal, Invalid, ToRetryRequirement};
pub use self::input::{input, Bound, Bytes, Input, MaybeString, Pattern, Span, String};
pub use self::reader::{BytesReader, Peek, Reader, StringReader};

// Re-exported types from core::fmt along with `DisplayBase` and `Write`.
// This is used crate wide with the exception of crate::display.
pub(crate) mod fmt {
    pub(crate) use crate::display::{DisplayBase, Write};
    pub(crate) use core::fmt::{Debug, Display, Error, Formatter, Result};
}
