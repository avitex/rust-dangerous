//! An introduction.
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
