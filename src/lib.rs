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
//!
//! # Feature flags
//!
//! | Feature          | Default     | Description
//! | ---------------- | ----------- | -------------------------------------------------- |
//! | `std`            | **Enabled** | Enables `std::error::Error` support and `alloc`    |
//! | `alloc`          | **Enabled** | Enables allocations.                               |
//! | `retry`          | **Enabled** | Enables retry support.                             |
//! | `simd`           | **Enabled** | Enables all supported SIMD optimisations.          |
//! | `unicode`        | **Enabled** | Enables improved unicode printing support.         |
//! | `full-backtrace` | **Enabled** | Enables collection of all contexts for `Expected`. |
//! | `zc`             | _Disabled_  | Enables `zc` crate support.                        |
//! | `nom`            | _Disabled_  | Enables `nom` crate error support.                 |
//! | `regex`          | _Disabled_  | Enables `regex` pattern support.                   |

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
mod support;
mod util;

pub mod display;
pub mod error;

pub use self::error::{Error, Expected, Fatal};
#[cfg(feature = "retry")]
pub use self::error::{Invalid, ToRetryRequirement};
pub use self::input::Bound;
pub use self::input::{input, Bytes, Input, MaybeString, Pattern, String};
pub use self::reader::{BytesReader, Peek, Reader, StringReader};

// Re-exported types from core::fmt along with `DisplayBase` and `Write`.
// This is used crate wide with the exception of crate::display.
pub(crate) mod fmt {
    pub(crate) use crate::display::{DisplayBase, Write};
    pub(crate) use core::fmt::{Debug, Display, Error, Formatter, Result};
}
