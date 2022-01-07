//! Usage guide.
//!
//! # Basic usage
//!
//! ```
//! use dangerous::{Input, Invalid};
//!
//! let input = dangerous::input(b"hello");
//! let result: Result<_, Invalid> = input.read_partial(|r| {
//!     r.read()
//! });
//!
//! assert_eq!(result, Ok((b'h', dangerous::input(b"ello"))));
//! ```
//! # Feature flags
//!
//! - `std` (**default feature**): Enables `std::error::Error` support and `alloc`.
//! - `alloc` (**default feature**): Enables allocations.
//! - `simd` (**default feature**): Enables all supported SIMD optimisations.
//! - `unicode` (**default feature**): Enables improved unicode printing support.
//! - `full-backtrace` (**default feature**): Enables collection of all contexts for `Expected`.
//!
//! **Third-party crate support (opt-in)**
//!
//! - `zc`: Enables `zc` crate support.
//! - `nom`: Enables `nom` crate error support.
//! - `regex`: Enables `regex` pattern support.

pub mod bounded;
pub mod external;
pub mod faq;
pub mod streaming;
