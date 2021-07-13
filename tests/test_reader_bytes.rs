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
    assert!(
        read_all_ok!(b"hello", |r| {
            let v = r.peek_u8()? == b'h';
            r.skip(5)?;
            Ok(v)
        })
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_u8_opt

#[test]
fn test_peek_u8_opt() {
    assert!(
        read_all_ok!(b"hello", |r| {
            let v = r.peek_u8_opt().map_or(false, |v| v == b'h');
            r.skip(5)?;
            Ok(v)
        })
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
