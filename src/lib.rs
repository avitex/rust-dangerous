//! Safely and explicitly handle user-generated aka `dangerous` data.
//!
//! # Basic usage
//!
//! ```rust
//! use dangerous::{Reader, Invalid};
//!
//! let input = dangerous::input(b"hello");
//!
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

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    // For byte slice to `Input` cast.
    unsafe_code,
    // For derived implementations.
    unused_qualifications,
    clippy::pedantic
)]
#![forbid(
    // TODO: box_pointers,
    anonymous_parameters,
    // TODO: missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    variant_size_differences,
    warnings
)]
#![allow(
    clippy::inline_always,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::type_repetition_in_bounds
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

mod error;
mod error_display;
mod input;
mod input_display;
mod reader;

#[cfg(any(feature = "std", feature = "alloc"))]
mod verbose_error;

pub use self::error::*;
pub use self::error_display::ErrorDisplay;
pub use self::input::{input, Input};
pub use self::input_display::InputDisplay;
pub use self::reader::Reader;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use self::verbose_error::VerboseError;
