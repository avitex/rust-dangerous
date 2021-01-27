#![allow(clippy::float_cmp)]
#![allow(clippy::unit_cmp)]

#[macro_use]
mod common;

use common::*;
use std::any::Any;
use std::str;

#[test]
fn test_reader_bytes_debug() {
    read_all!(b"hello", |r| {
        assert_eq!(
            format!("{:?}", r),
            "Reader { input: Bytes { bound: Start, bytes: [68 65 6c 6c 6f] } }"
        );
        r.consume(b"hello")
    })
    .unwrap();
}

#[test]
fn test_reader_bytes_pretty_debug() {
    read_all!(b"hello", |r| {
        assert_eq!(
            format!("{:#?}\n", r),
            indoc! {r#"
                Reader {
                    input: Bytes {
                        bound: Start,
                        bytes: "hello",
                    },
                }
            "#}
        );
        r.consume(b"hello")
    })
    .unwrap();
}

#[test]
fn test_read_nums() {
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

    assert_eq!(read_all!(&[0x1], |r| r.read_u8()).unwrap(), 1);
    assert_eq!(read_all!(&[0b0000_0001], |r| r.read_i8()).unwrap(), 1);
    assert_eq!(read_all!(&[0b1000_0000], |r| r.read_i8()).unwrap(), -128);

    validate_read_num!(u16, le: read_u16_le, be: read_u16_be);
    validate_read_num!(i16, le: read_i16_le, be: read_i16_be);
    validate_read_num!(u32, le: read_u32_le, be: read_u32_be);
    validate_read_num!(i32, le: read_i32_le, be: read_i32_be);
    validate_read_num!(u64, le: read_u64_le, be: read_u64_be);
    validate_read_num!(i64, le: read_i64_le, be: read_i64_be);
    validate_read_num!(u128, le: read_u128_le, be: read_u128_be);
    validate_read_num!(i128, le: read_i128_le, be: read_i128_be);
    validate_read_num!(f32, le: read_f32_le, be: read_f32_be);
    validate_read_num!(f64, le: read_f64_le, be: read_f64_be);
}

#[test]
fn test_at_end() {
    assert_eq!(
        read_all!(b"hello", |r| {
            r.consume(b"hello")?;
            Ok(r.at_end())
        })
        .unwrap(),
        true
    );
}

#[test]
fn test_context() {
    let err = read_all!(b"hello", |r| { r.context("bob", |r| r.consume(b"world")) }).unwrap_err();
    #[cfg(feature = "full-context")]
    assert_eq!(err.context_stack().count(), 3);
    #[cfg(not(feature = "full-context"))]
    assert_eq!(err.context_stack().count(), 1);
    err.context_stack().walk(&mut |i, c| {
        // i == 1 is an operation context which cannot be downcast
        if i == 2 {
            let c = Any::downcast_ref::<&'static str>(c.as_any());
            assert_eq!(c, Some(&"bob"));
        }
        if i == 3 {
            let c = Any::downcast_ref::<ExpectedContext>(c.as_any());
            assert!(c.is_some());
        }
        assert!(i != 5);
        true
    });
}

#[test]
fn test_skip() {
    assert_eq!(read_all!(b"hello", |r| { r.skip(5) }).unwrap(), ());
}

#[test]
fn test_skip_while() {
    read_all!(b"hello!", |r| {
        r.skip_while(|c| c.is_ascii_alphabetic());
        r.skip(1)
    })
    .unwrap();
}

#[test]
fn test_try_skip_while() {
    read_all!(b"hello!", |r| {
        r.try_skip_while(|c| Ok(c.is_ascii_alphabetic()))?;
        r.skip(1)
    })
    .unwrap()
}

#[test]
fn test_take() {
    assert_eq!(
        read_all!(b"hello", |r| { r.take(5) }).unwrap(),
        b"hello"[..]
    );
}

#[test]
fn test_take_remaining() {
    assert_eq!(
        read_all!(b"hello", |r| { Ok(r.take_remaining()) }).unwrap(),
        b"hello"[..]
    );
}

#[test]
fn test_take_remaining_str() {
    assert_eq!(
        read_all!(b"hello!", |r| { r.take_remaining_str() }).unwrap(),
        "hello!"[..]
    );
}

#[test]
fn test_take_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.take_while(|c| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        b"hello"[..]
    );
}

#[test]
fn test_try_take_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_take_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        b"hello"[..]
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
        input!(str::from_utf8(&[0b0111_1111]).unwrap())
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
fn test_take_consumed() {
    assert_eq!(
        read_all!(b"hello", |r| {
            Ok(r.take_consumed(|r| {
                let _ = r.take_remaining();
            }))
        })
        .unwrap(),
        b"hello"[..]
    );
}

#[test]
fn test_try_take_consumed() {
    assert_eq!(
        read_all!(b"hello", |r| {
            r.try_take_consumed(|r| r.consume(b"hello"))
        })
        .unwrap(),
        b"hello"[..]
    );
}

#[test]
fn test_peek() {
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.peek(4)? == b"hell"[..];
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
    let _ = read_all!(b"hello", |r| {
        let _ = r.peek(6)?;
        r.skip(5)
    })
    .unwrap_err();
}

#[test]
fn test_peek_opt() {
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.peek_opt(4).map_or(false, |v| v == b"hell"[..]);
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}

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

#[test]
fn test_peek_eq() {
    let _ = read_partial!(b"helloworld", |r| {
        assert!(r.peek_eq(b"helloworld"));
        assert!(r.peek_eq(b"hello"));
        assert!(!r.peek_eq(b"no"));
        assert!(!r.peek_eq(b"helloworld!"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_consume() {
    // Valid
    assert_eq!(
        read_all!(b"hello", |r| { r.consume(b"hello") }).unwrap(),
        ()
    );
    // Invalid
    assert_eq!(
        read_all!(b"hell", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(1)
    );
    assert_eq!(
        read_all!(b"abcde", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"abc", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
}

#[test]
fn test_consume_opt() {
    // Valid
    assert!(read_all!(b"hello", |r| { Ok(r.consume_opt(b"hello")) }).unwrap());
    // Invalid
    assert!(!read_all!(b"abc", |r| {
        let v = r.consume_opt(b"hello");
        r.skip(3)?;
        Ok(v)
    })
    .unwrap());
}

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

#[test]
fn test_verify() {
    // Valid
    read_all!(b"1", |r| { r.verify("value", |r| r.consume_opt(b"1")) }).unwrap();
    // Invalid
    let _ = read_all!(b"1", |r| { r.verify("value", |r| r.consume_opt(b"2")) }).unwrap_err();
}

#[test]
fn test_try_verify() {
    // Valid
    read_all!(b"1", |r| {
        r.try_verify("value", |r| Ok(r.consume_opt(b"1")))
    })
    .unwrap();
    // Invalid
    let _ = read_all!(b"1", |r| {
        r.try_verify("value", |r| Ok(r.consume_opt(b"2")))
    })
    .unwrap_err();
}

#[test]
fn test_expect() {
    // Valid
    read_all!(b"1", |r| {
        r.expect("value", |r| Some(r.consume_opt(b"1")))
    })
    .unwrap();
    // Invalid
    let _ = read_all!(b"", |r| { r.expect("value", |_| Option::<()>::None) }).unwrap_err();
}

#[test]
fn try_try_expect() {
    // Valid
    read_all!(b"", |r| { r.try_expect("value", |_| Ok(Some(()))) }).unwrap();
    // Invalid
    let _ = read_all!(b"", |r| {
        r.try_expect("value", |_| Ok(Option::<()>::None))
    })
    .unwrap_err();
}

#[test]
fn try_expect_erased() {
    // Valid
    read_all!(b"", |r| {
        r.try_expect_erased("value", |_| Result::<(), Fatal>::Ok(()))
    })
    .unwrap();
    // Invalid
    let _ = read_all!(b"", |r| {
        r.try_expect_erased("value", |_| Result::<(), Fatal>::Err(Fatal))
    })
    .unwrap_err();
}

#[test]
fn test_recover() {
    read_all!(b"", |r| {
        r.recover(|r| r.take(1));
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_recover_if() {
    // Valid
    read_all!(b"", |r| { r.recover_if(|r| { r.take(1) }, |_| true) }).unwrap();
    // Invalid
    let _ = read_all!(b"", |r| { r.recover_if(|r| { r.take(1) }, |_| false) }).unwrap_err();
}

#[test]
fn test_error() {
    assert!(read_all!(b"", |r| {
        r.try_expect_erased("value", |r| r.error(|r: &mut BytesReader<Fatal>| r.take(1)))
    })
    .unwrap_err()
    .is_fatal())
}
