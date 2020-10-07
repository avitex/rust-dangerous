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

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![deny(
    // Exceptions: Byte slice to `Input` cast, str cast for section
    unsafe_code,
    // Exception: Derived implementations
    unused_qualifications,
    // Exception: `error::Value`
    variant_size_differences,
)]
#![forbid(
    clippy::pedantic,
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

mod input;
mod reader;
mod string;

pub mod display;
pub mod error;

pub use self::error::{Error, Expected, FromContext, FromExpected, Invalid, ToRetryRequirement};
pub use self::input::{input, Input};
pub use self::reader::Reader;

mod util {
    use core::ops::Range;

    // FIXME: use https://github.com/rust-lang/rust/issues/65807 when stable in 1.48
    #[inline(always)]
    pub(super) fn slice_ptr_range<T>(slice: &[T]) -> Range<*const T> {
        let start = slice.as_ptr();
        // note: will never wrap, we are just escaping the use of unsafe.
        let end = slice.as_ptr().wrapping_add(slice.len());
        debug_assert!(start <= end);
        start..end
    }

    #[inline(always)]
    pub(super) fn is_sub_slice<T>(parent: &[T], sub: &[T]) -> bool {
        let parent_bounds = slice_ptr_range(parent);
        let sub_bounds = slice_ptr_range(sub);
        parent_bounds.start <= sub_bounds.start && parent_bounds.end >= sub_bounds.end
    }
}
