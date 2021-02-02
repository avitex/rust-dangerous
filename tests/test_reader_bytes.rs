#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// Test debug

#[test]
fn test_bytes_input_debug() {
    read_all!(b"hello", |r| {
        assert_eq!(
            format!("{:?}", r),
            "Reader { input: Bytes { bound: Start, value: [68 65 6c 6c 6f] } }"
        );
        r.consume(b"hello")
    })
    .unwrap();
}

#[test]
fn test_bytes_input_pretty_debug() {
    read_all!(b"hello", |r| {
        assert_eq!(
            format!("{:#?}\n", r),
            indoc! {r#"
                Reader {
                    input: Bytes {
                        bound: Start,
                        value: "hello",
                    },
                }
            "#}
        );
        r.consume(b"hello")
    })
    .unwrap();
}

///////////////////////////////////////////////////////////////////////////////
// Test consuming

#[test]
fn test_consume_u8() {
    // Valid
    read_all!(b"1", |r| { r.consume(b'1') }).unwrap();
    // Invalid
    assert_eq!(
        read_all!(b"1", |r| { r.consume(b'2') })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"", |r| { r.consume(b'1') })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

#[test]
fn test_consume_u8_opt() {
    // Valid
    assert!(read_all!(b"1", |r| { Ok(r.consume_opt(b'1')) }).unwrap());
    // Invalid
    assert!(!read_all!(b"1", |r| {
        let v = r.consume_opt(b'2');
        r.skip(1)?;
        Ok(v)
    })
    .unwrap());
}

///////////////////////////////////////////////////////////////////////////////
// Test peeking

#[test]
fn test_peek_u8() {
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.peek_u8()? == b'h';
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}

#[test]
fn test_peek_u8_opt() {
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.peek_u8_opt().map_or(false, |v| v == b'h');
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}

///////////////////////////////////////////////////////////////////////////////
// Test UTF-8

#[test]
fn test_try_take_str_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_take_str_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        "hello"[..]
    );
}

#[test]
fn test_try_skip_str_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_skip_str_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        5
    );
}

#[test]
fn test_take_str_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.take_str_while(|c| c.is_ascii_alphabetic())?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        "hello"[..]
    );
}

#[test]
fn test_skip_str_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.skip_str_while(|c| c.is_ascii_alphabetic())?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        5
    );
}

#[test]
fn test_take_str_while_utf8_retry() {
    // Length 1
    assert_eq!(
        read_all!(&[0b0111_1111], |r| r.take_str_while(|_| true)).unwrap(),
        input!(core::str::from_utf8(&[0b0111_1111]).unwrap())
    );
    // Length 2
    let err = read_all!(&[0b1101_1111], |r| r.take_str_while(|_| true)).unwrap_err();
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(1));
    // Length 3
    let err = read_all!(&[0b1110_1111], |r| r.take_str_while(|_| true)).unwrap_err();
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(2));
    // Invalid
    let err = read_all!(&[0b1111_0111], |r| r.take_str_while(|_| true)).unwrap_err();
    assert_eq!(err.to_retry_requirement(), None);
}

///////////////////////////////////////////////////////////////////////////////
// Test numbers

macro_rules! test_can_read_num {
    (
        type: $ty:ty,
        name: $test_name:ident,
        read_le: $read_le:ident,
        read_be: $read_be:ident
    ) => {
        #[test]
        #[allow(clippy::float_cmp)]
        fn $test_name() {
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
        }
    };
}

#[test]
fn test_can_read_u8() {
    assert_eq!(read_all!(&[0x0], |r| r.read_u8()).unwrap(), 0);
    assert_eq!(read_all!(&[0x1], |r| r.read_u8()).unwrap(), 1);
}

#[test]
fn test_can_read_i8() {
    assert_eq!(read_all!(&[0x0], |r| r.read_i8()).unwrap(), 0);
    assert_eq!(read_all!(&[0b0000_0001], |r| r.read_i8()).unwrap(), 1);
    assert_eq!(read_all!(&[0b1000_0000], |r| r.read_i8()).unwrap(), -128);
}

test_can_read_num!(
    type: u16,
    name: test_can_read_u16,
    read_le: read_u16_le,
    read_be: read_u16_be
);

test_can_read_num!(
    type: i16,
    name: test_can_read_i16,
    read_le: read_i16_le,
    read_be: read_i16_be
);

test_can_read_num!(
    type: u32,
    name: test_can_read_u32,
    read_le: read_u32_le,
    read_be: read_u32_be
);

test_can_read_num!(
    type: i32,
    name: test_can_read_i32,
    read_le: read_i32_le,
    read_be: read_i32_be
);

test_can_read_num!(
    type: u64,
    name: test_can_read_u64,
    read_le: read_u64_le,
    read_be: read_u64_be
);

test_can_read_num!(
    type: i64,
    name: test_can_read_i64,
    read_le: read_i64_le,
    read_be: read_i64_be
);

test_can_read_num!(
    type: u128,
    name: test_can_read_u128,
    read_le: read_u128_le,
    read_be: read_u128_be
);

test_can_read_num!(
    type: i128,
    name: test_can_read_i128,
    read_le: read_i128_le,
    read_be: read_i128_be
);

test_can_read_num!(
    type: f32,
    name: test_can_read_f32,
    read_le: read_f32_le,
    read_be: read_f32_be
);

test_can_read_num!(
    type: f64,
    name: test_can_read_f64,
    read_le: read_f64_le,
    read_be: read_f64_be
);
