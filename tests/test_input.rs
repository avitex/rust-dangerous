#![allow(clippy::unit_cmp)]

#[macro_use]
mod common;

use dangerous::error::{Expected, RetryRequirement, ToRetryRequirement};

#[test]
fn test_as_dangerous() {
    assert_eq!(input!(b"").as_dangerous(), b"");
    assert_eq!(input!(b"hello").as_dangerous(), b"hello");
}

#[test]
fn test_to_dangerous_non_empty() {
    // Valid
    assert_eq!(
        input!(b"hello")
            .to_dangerous_non_empty::<Expected>()
            .unwrap(),
        b"hello"
    );
    // Invalid
    input!(b"")
        .to_dangerous_non_empty::<Expected>()
        .unwrap_err();
}

#[test]
fn test_as_dangerous_str() {
    // Valid
    assert_eq!(input!(b"").to_dangerous_str::<Expected>().unwrap(), "");
    assert_eq!(
        input!(b"hello").to_dangerous_str::<Expected>().unwrap(),
        "hello"
    );
    // Invalid
    input!(b"\xff").to_dangerous_str::<Expected>().unwrap_err();
}

#[test]
fn test_to_dangerous_non_empty_str() {
    // Valid
    assert_eq!(
        input!(b"hello")
            .to_dangerous_non_empty_str::<Expected>()
            .unwrap(),
        "hello"
    );
    // Invalid
    input!(b"")
        .to_dangerous_non_empty_str::<Expected>()
        .unwrap_err();
    input!(b"\xff")
        .to_dangerous_non_empty_str::<Expected>()
        .unwrap_err();
}

#[test]
fn test_is_within() {
    let bytes = [0u8; 64];

    // Within
    let parent = input!(&bytes[16..32]);
    let child = input!(&bytes[20..24]);
    assert!(child.is_within(parent));
    assert!(parent.is_within(parent));

    // Left out of bound
    let parent = input!(&bytes[16..32]);
    let child = input!(&bytes[15..24]);
    assert!(!child.is_within(parent));

    // Right out of bound
    let parent = input!(&bytes[16..32]);
    let child = input!(&bytes[20..33]);
    assert!(!child.is_within(parent));

    // Both out of bound
    let parent = input!(&bytes[16..32]);
    let child = input!(&bytes[15..33]);
    assert!(!child.is_within(parent));
}

#[test]
fn test_to_dangerous_str_expected_length() {
    // Length 1
    input!(&[0b0111_1111])
        .to_dangerous_str::<Expected>()
        .unwrap();
    // Length 2
    let err = input!(&[0b1101_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(1));
    // Length 3
    let err = input!(&[0b1110_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.to_retry_requirement(), RetryRequirement::new(2));
    // Invalid
    let err = input!(&[0b1111_0111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.to_retry_requirement(), None);
}

#[test]
fn test_read_all() {
    // Valid
    assert_eq!(
        read_all!(b"hello", |r| { r.consume(b"hello") }).unwrap(),
        ()
    );
    assert_eq!(
        read_all!(b"hello", |r| { r.take(5) }).unwrap(),
        input!(b"hello")
    );
    // Invalid
    assert_eq!(
        read_all!(b"hello", |r| { r.consume(b"hell") })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"hello", |r| { r.take(4) })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"hello", |r| { r.take(10) })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(5)
    );
}

#[test]
fn test_read_partial() {
    // Valid
    assert_eq!(
        read_partial!(b"hello", |r| { r.consume(b"hello") }).unwrap(),
        ((), input!(b""))
    );
    assert_eq!(
        read_partial!(b"hello", |r| { r.take(5) }).unwrap(),
        (input!(b"hello"), input!(b""))
    );
    assert_eq!(
        read_partial!(b"hello", |r| { r.consume(b"hell") }).unwrap(),
        ((), input!(b"o"))
    );
    // Invalid
    assert_eq!(
        read_partial!(b"hello", |r| { r.take(10) })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(5)
    );
}

#[test]
fn test_read_infallible() {
    assert_eq!(
        read_infallible!(b"hello", |r| { r.take_while(|c| c.is_ascii_alphabetic()) }),
        (input!(b"hello"), input!(b""))
    );
    assert_eq!(
        read_infallible!(b"hello1", |r| { r.take_while(|c| c.is_ascii_alphabetic()) }),
        (input!(b"hello"), input!(b"1"))
    );
}
