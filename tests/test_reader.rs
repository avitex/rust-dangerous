macro_rules! assert_read_eq {
    ($input:expr, $read_fn:expr, $expect:expr) => {{
        let input = &$input[..];
        let mut reader = dangerous::input(input).reader();
        assert!(reader.read_all($read_fn).is_ok());
    }};
}

macro_rules! validate_read_num {
    ($ty:ty, le: $read_le:ident, be: $read_be:ident) => {
        assert_read_eq!(
            <$ty>::to_le_bytes(<$ty>::MIN),
            |r: &mut dangerous::Reader<'_>| Ok(r.$read_le()?),
            Ok(<$ty>::MIN)
        );
        assert_read_eq!(
            <$ty>::to_be_bytes(<$ty>::MIN),
            |r: &mut dangerous::Reader<'_>| Ok(r.$read_be()?),
            Ok(<$ty>::MIN)
        );
        assert_read_eq!(
            <$ty>::to_le_bytes(<$ty>::MAX),
            |r: &mut dangerous::Reader<'_>| Ok(r.$read_le()?),
            Ok(<$ty>::MAX)
        );
        assert_read_eq!(
            <$ty>::to_be_bytes(<$ty>::MAX),
            |r: &mut dangerous::Reader<'_>| Ok(r.$read_be()?),
            Ok(<$ty>::MAX)
        );
    };
}

#[test]
fn read_nums() {
    validate_read_num!(i8, le: read_i8_le, be: read_i8_be);
    validate_read_num!(u16, le: read_u16_le, be: read_u16_be);
    validate_read_num!(i16, le: read_i16_le, be: read_i16_be);
    validate_read_num!(u32, le: read_u32_le, be: read_u32_be);
    validate_read_num!(i32, le: read_i32_le, be: read_i32_be);
    validate_read_num!(u64, le: read_u64_le, be: read_u64_be);
    validate_read_num!(i64, le: read_i64_le, be: read_i64_be);
    validate_read_num!(f32, le: read_f32_le, be: read_f32_be);
    validate_read_num!(f64, le: read_f64_le, be: read_f64_be);
}
