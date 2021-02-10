#![allow(unused_macros)]

pub use dangerous::{error::*, *};
pub use indoc::indoc;
pub use paste::paste;

macro_rules! assert_str_eq {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual[..];
        let expected = &$expected[..];
        if actual != expected {
            panic!(
                indoc! {"
                string not expected value:
                ============================EXPECTED==========================
                {}
                =============================ACTUAL===========================
                {}
                ==============================DIFF============================
                {}
                ==============================================================
            "},
                expected,
                actual,
                colored_diff::PrettyDifference { expected, actual },
            );
        }
    }};
}

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

macro_rules! read_all_ok {
    ($input:expr, $read_fn:expr) => {
        read_all!($input, $read_fn).unwrap()
    };
}

macro_rules! read_all_err {
    ($input:expr, $read_fn:expr) => {
        read_all!($input, $read_fn).unwrap_err()
    };
}

macro_rules! read_partial_ok {
    ($input:expr, $read_fn:expr) => {
        read_partial!($input, $read_fn).unwrap()
    };
}

macro_rules! read_partial_err {
    ($input:expr, $read_fn:expr) => {
        read_partial!($input, $read_fn).unwrap_err()
    };
}
