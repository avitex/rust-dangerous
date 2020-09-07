#[macro_use]
mod common;

use dangerous::{Error, Expected};

use core::num::NonZeroUsize;

#[test]
fn as_dangerous() {
    assert_eq!(input!(b"").as_dangerous(), b"");
    assert_eq!(input!(b"hello").as_dangerous(), b"hello");
}

#[test]
fn to_dangerous_non_empty() {
    // Valid
    assert_eq!(
        input!(b"hello").to_dangerous_non_empty().unwrap(),
        b"hello"
    );
    // Invalid
    input!(b"").to_dangerous_non_empty().unwrap_err();
}

#[test]
fn as_dangerous_str() {
    // Valid
    assert_eq!(
        input!(b"")
            .to_dangerous_str::<Expected>()
            .unwrap(),
        ""
    );
    assert_eq!(
        input!(b"hello")
            .to_dangerous_str::<Expected>()
            .unwrap(),
        "hello"
    );
    // Invalid
    input!(b"\xff")
        .to_dangerous_str::<Expected>()
        .unwrap_err();
}

#[test]
fn to_dangerous_non_empty_str() {
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
fn is_within() {
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
fn to_dangerous_str_expected_length() {
    // Length 1
    input!(&[0b0111_1111])
        .to_dangerous_str::<Expected>()
        .unwrap();
    // Length 2
    let err = input!(&[0b1101_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), NonZeroUsize::new(1));
    // Length 3
    let err = input!(&[0b1110_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), NonZeroUsize::new(2));
    // Invalid
    let err = input!(&[0b1111_0111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), None);
}
