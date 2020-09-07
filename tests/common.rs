#![allow(unused_macros)]

macro_rules! input {
    ($input:expr) => {
        dangerous::input($input.as_ref())
    }
}

macro_rules! reader {
    ($input:expr) => {
        input!($input).reader::<dangerous::Expected>()
    }
}

macro_rules! assert_input_display_eq {
    ($input:expr, $format:expr, $expected:expr) => {
        assert_eq!(
            format!($format, input!($input)),
            $expected
        );
    };
}

macro_rules! assert_read_all_eq {
    ($input:expr, $read_fn:expr, $expect:expr) => {{
        let input = $input;
        let mut reader = reader!(input);
        assert_eq!(reader.read_all($read_fn).expect("value"), $expect);
    }};
}

macro_rules! validate_read_num {
    ($ty:ty, le: $read_le:ident, be: $read_be:ident) => {
        assert_read_all_eq!(
            <$ty>::to_le_bytes(<$ty>::MIN),
            |r| Ok(r.$read_le()?),
            <$ty>::MIN
        );
        assert_read_all_eq!(
            <$ty>::to_be_bytes(<$ty>::MIN),
            |r| Ok(r.$read_be()?),
            <$ty>::MIN
        );
        assert_read_all_eq!(
            <$ty>::to_le_bytes(<$ty>::MAX),
            |r| Ok(r.$read_le()?),
            <$ty>::MAX
        );
        assert_read_all_eq!(
            <$ty>::to_be_bytes(<$ty>::MAX),
            |r| Ok(r.$read_be()?),
            <$ty>::MAX
        );
    };
}
