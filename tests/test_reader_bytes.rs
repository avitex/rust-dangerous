#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// Test debug

#[test]
fn test_bytes_input_debug() {
    read_all_ok!(b"hello", |r| {
        assert_eq!(
            format!("{:?}", r),
            "Reader { input: Bytes { bound: Start, value: [68 65 6c 6c 6f] } }"
        );
        r.consume(b"hello")
    });
}

#[test]
fn test_bytes_input_pretty_debug() {
    read_all_ok!(b"hello", |r| {
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
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume (u8)

#[test]
fn test_consume_u8_exact_same() {
    read_all_ok!(b"1", |r| { r.consume(b'1') });
}

#[test]
fn test_consume_u8_same_len_different_value() {
    assert_eq!(
        read_all_err!(b"1", |r| { r.consume(b'2') }).to_retry_requirement(),
        None
    );
}

#[test]
fn test_consume_u8_different_len_and_value() {
    assert_eq!(
        read_all_err!(b"", |r| { r.consume(b'1') }).to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume_opt (u8)

#[test]
fn test_consume_opt_u8_true() {
    assert!(read_all_ok!(b"1", |r| { Ok(r.consume_opt(b'1')) }));
}

#[test]
fn test_consume_opt_u8_false() {
    assert!(!read_all_ok!(b"1", |r| {
        let v = r.consume_opt(b'2');
        r.skip(1)?;
        Ok(v)
    }));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_u8

#[test]
fn test_peek_u8() {
    assert_eq!(
        read_all_ok!(b"hello", |r| {
            let v = r.peek_u8()? == b'h';
            r.skip(5)?;
            Ok(v)
        }),
        true
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_u8_opt

#[test]
fn test_peek_u8_opt() {
    assert_eq!(
        read_all_ok!(b"hello", |r| {
            let v = r.peek_u8_opt().map_or(false, |v| v == b'h');
            r.skip(5)?;
            Ok(v)
        }),
        true
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::skip_str_while

#[test]
fn test_skip_str_while() {
    read_all_ok!(b"hello!", |r| {
        r.skip_str_while(|c| c.is_ascii_alphabetic())?;
        r.skip(1)?;
        Ok(())
    })
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take_str_while

#[test]
fn test_take_str_while() {
    assert_eq!(
        read_all_ok!(b"hello!", |r| {
            let v = r.take_str_while(|c| c.is_ascii_alphabetic())?;
            r.skip(1)?;
            Ok(v)
        }),
        "hello"[..]
    );
}

#[test]
fn test_take_str_while_utf8_retry() {
    // Length 1
    assert_eq!(
        read_all_ok!(&[0b0111_1111], |r| r.take_str_while(|_| true)),
        input!(core::str::from_utf8(&[0b0111_1111]).unwrap())
    );
    // Length 2
    let err = read_all_err!(&[0b1101_1111], |r| r.take_str_while(|_| true));
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(1));
    // Length 3
    let err = read_all_err!(&[0b1110_1111], |r| r.take_str_while(|_| true));
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(2));
    // Invalid
    let err = read_all_err!(&[0b1111_0111], |r| r.take_str_while(|_| true));
    assert_eq!(err.to_retry_requirement(), None);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_skip_str_while

#[test]
fn test_try_skip_str_while() {
    read_all_ok!(b"hello!", |r| {
        r.try_skip_str_while(|c| Ok(c.is_ascii_alphabetic()))?;
        r.skip(1)?;
        Ok(())
    })
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_take_str_while

#[test]
fn test_try_take_str_while() {
    assert_eq!(
        read_all_ok!(b"hello!", |r| {
            let v = r.try_take_str_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        }),
        "hello"[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// Test numbers

macro_rules! test_can_read_num {
    ($($ty:ident),*) => {
        $(
            paste! {
                #[test]
                #[allow(clippy::float_cmp)]
                fn [<test_can_read_ $ty>]() {
                    assert_eq!(
                        read_all_ok!(<$ty>::to_le_bytes(<$ty>::MIN), |r| r.[<read_ $ty _le>]()),
                        <$ty>::MIN
                    );
                    assert_eq!(
                        read_all_ok!(<$ty>::to_be_bytes(<$ty>::MIN), |r| r.[<read_ $ty _be>]()),
                        <$ty>::MIN
                    );
                    assert_eq!(
                        read_all_ok!(<$ty>::to_le_bytes(<$ty>::MAX), |r| r.[<read_ $ty _le>]()),
                        <$ty>::MAX
                    );
                    assert_eq!(
                        read_all_ok!(<$ty>::to_be_bytes(<$ty>::MAX), |r| r.[<read_ $ty _be>]()),
                        <$ty>::MAX
                    );
                }
            }
        )*
    };
}

#[test]
fn test_can_read_u8() {
    assert_eq!(read_all_ok!(&[0x0], |r| r.read_u8()), 0);
    assert_eq!(read_all_ok!(&[0x1], |r| r.read_u8()), 1);
}

#[test]
fn test_can_read_i8() {
    assert_eq!(read_all_ok!(&[0x0], |r| r.read_i8()), 0);
    assert_eq!(read_all_ok!(&[0b0000_0001], |r| r.read_i8()), 1);
    assert_eq!(read_all_ok!(&[0b1000_0000], |r| r.read_i8()), -128);
}

test_can_read_num!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);
