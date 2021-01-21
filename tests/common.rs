#![allow(unused_macros)]

pub use dangerous::{error::*, *};
pub use indoc::indoc;

macro_rules! input {
    ($input:expr) => {
        dangerous::input(&$input)
    };
}

macro_rules! read_all {
    ($input:expr, $read_fn:expr) => {
        input!($input).read_all::<_, _, dangerous::Expected>($read_fn)
    };
}

macro_rules! read_partial {
    ($input:expr, $read_fn:expr) => {
        input!($input).read_partial::<_, _, dangerous::Expected>($read_fn)
    };
}

macro_rules! read_infallible {
    ($input:expr, $read_fn:expr) => {
        input!($input).read_infallible($read_fn)
    };
}

macro_rules! assert_input_display_eq {
    ($input:expr, $format:expr, $expected:expr) => {
        assert_eq!(
            format!($format, input!(<&[u8]>::from($input.as_ref()))),
            $expected
        );
    };
}
