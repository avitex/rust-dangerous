//! Safely and explicitly parse untrusted aka `dangerous` data.
//!
//! # Basic usage
//!
//! ```
//! use dangerous::{Input, Invalid};
//!
//! let input = dangerous::input(b"hello");
//! let result: Result<_, Invalid> = input.read_partial(|r| {
//!     r.read_u8()
//! });
//!
//! assert_eq!(result, Ok((b'h', dangerous::input(b"ello"))));
//! ```

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "unstable-doc-cfg", feature(doc_cfg))]
#![forbid(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    future_incompatible
)]
#![deny(
    unused,
    rustdoc,
    rust_2018_idioms,
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

#[macro_use]
mod macros;

mod input;
mod reader;
mod util;

pub mod display;
pub mod error;

pub use self::error::{Error, Expected, Fatal, FromExpected, WithContext};
#[cfg(feature = "retry")]
#[cfg_attr(feature = "unstable-doc-cfg", doc(cfg(feature = "retry")))]
pub use self::error::{Invalid, ToRetryRequirement};
#[cfg(feature = "retry")]
#[cfg_attr(feature = "unstable-doc-cfg", doc(cfg(feature = "retry")))]
pub use self::input::Bound;
pub use self::input::{input, Bytes, Input, MaybeString, String};
pub use self::reader::{BytesReader, Reader, StringReader};

pub(crate) mod fmt {
    pub(crate) use crate::display::{DisplayBase, Write};
    pub(crate) use core::fmt::{Debug, Display, Error, Formatter, Result};
}
