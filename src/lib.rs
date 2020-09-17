//! Safely and explicitly handle user-generated aka `dangerous` data.
//!
//! # Basic usage
//!
//! ```rust
//! use dangerous::Invalid;
//!
//! let input = dangerous::input(b"hello");
//! let result: Result<_, Invalid> = input.read_partial(|r| {
//!     r.read_u8()
//! });
//!
//! assert_eq!(result, Ok((b'h', dangerous::input(b"ello"))));
//! ```
//!
//! # Guarantees
//!
//! - Zero panics.
//! - Zero heap-allocations.
//!
//! Once an input is cast with `to_dangerous*`, these guarantees end.
//!
//! # Safety
//!
//! This library has one instance of `unsafe` required for wrapping a byte slice
//! into the `Input` DST.
//!
//! **No other instances of `unsafe` are permitted.**

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    // For byte slice to `Input` cast.
    unsafe_code,
    // For derived implementations.
    unused_qualifications,
    variant_size_differences,
    clippy::pedantic
)]
#![forbid(
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
#![allow(
    clippy::inline_always,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::type_repetition_in_bounds
)]

#[cfg(feature = "full-context")]
extern crate alloc;

#[macro_use]
mod macros;

pub mod error;
mod input;
mod reader;
mod utils;

pub use self::error::{
    Error, Expected, FromExpected, Invalid, RetryRequirement, ToRetryRequirement,
};
pub use self::input::{input, Input, InputDisplay};
pub use self::reader::Reader;
