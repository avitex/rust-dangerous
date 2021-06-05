//! Frequently asked questions (ie. Why can't I do this?)
//!
//! ## Why isn't parsing numbers from string representation supported?
//!
//! Numbers have many different types of representations in string formats and
//! can be parsed in many different ways. While Rust's standard library supports
//! parsing numbers from strings, it doesn't support parsing them directly from
//! bytes. Dangerous seeks to provide a foundation for other parsing libraries
//! leaving the representation and the method of parsing as a downstream
//! concern.
//!
//! Dangerous does implement support via [`error::External`] for the errors
//! returned by the standard library's `from_str_radix` functions. See
//! [`guide::external`] for more information.
//!
//! [`guide::external`]: crate::guide::external  
//! [`error::External`]: crate::error::External
