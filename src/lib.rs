//! Safely and explicitly parse untrusted aka `dangerous` data.
//!
//! # Basic usage
//!
//! ```
//! use dangerous::Invalid;
//!
//! let input = dangerous::input(b"hello");
//! let result: Result<_, Invalid> = input.read_partial(|r| {
//!     r.read_u8()
//! });
//!
//! assert_eq!(result, Ok((b'h', dangerous::input(b"ello"))));
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(
    rust_2018_idioms,
    anonymous_parameters,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    warnings
)]
// Exceptions: Derived implementations
// FIXME: remove once https://github.com/rust-lang/rust/issues/71898 is fixed
#![deny(unused_qualifications)]
// Exceptions: See below allow.
#![deny(clippy::pedantic)]
#![allow(
    clippy::inline_always,
    clippy::single_match_else,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::type_repetition_in_bounds
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

pub use self::error::{
    Error, Expected, Fatal, FromContext, FromExpected, Invalid, ToRetryRequirement,
};
pub use self::input::{input, Input};
pub use self::reader::Reader;
