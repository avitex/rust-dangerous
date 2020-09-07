#![allow(unused_macros)]

macro_rules! input {
    ($input:expr) => {
        dangerous::input($input.as_ref())
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

macro_rules! assert_input_display_eq {
    ($input:expr, $format:expr, $expected:expr) => {
        assert_eq!(format!($format, input!($input)), $expected);
    };
}

macro_rules! validate_read_num {
    ($ty:ty, le: $read_le:ident, be: $read_be:ident) => {
        assert_eq!(
            read_all!(<$ty>::to_le_bytes(<$ty>::MIN), |r| r.$read_le()).unwrap(),
            <$ty>::MIN
        );
        assert_eq!(
            read_all!(<$ty>::to_be_bytes(<$ty>::MIN), |r| r.$read_be()).unwrap(),
            <$ty>::MIN
        );
        assert_eq!(
            read_all!(<$ty>::to_le_bytes(<$ty>::MAX), |r| r.$read_le()).unwrap(),
            <$ty>::MAX
        );
        assert_eq!(
            read_all!(<$ty>::to_be_bytes(<$ty>::MAX), |r| r.$read_be()).unwrap(),
            <$ty>::MAX
        );
    };
}
