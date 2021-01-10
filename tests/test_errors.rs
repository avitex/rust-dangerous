#[macro_use]
mod common;

use dangerous::error::RetryRequirement;
use dangerous::{Expected, Fatal, Invalid, ToRetryRequirement};

#[test]
fn test_fatal() {
    let error = input!(b"")
        .read_all::<_, _, Fatal>(|r| r.consume(b"1"))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(format!("{}", error), "invalid input");
    assert_eq!(format!("{:?}", error), "Fatal");
}

#[test]
fn test_invalid_retry_1_more() {
    let error = input!(b"")
        .read_all::<_, _, Invalid>(|r| r.consume(b"1"))
        .unwrap_err();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(1));
    assert_eq!(
        format!("{}", error),
        "invalid input: needs 1 byte more to continue processing"
    );
    assert_eq!(
        format!("{:?}", error),
        "Invalid { retry_requirement: Some(RetryRequirement(1)) }"
    );
}

#[test]
fn test_invalid_retry_2_more() {
    let error = input!(b"")
        .read_all::<_, _, Invalid>(|r| r.consume(b"12"))
        .unwrap_err();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_eq!(
        format!("{}", error),
        "invalid input: needs 2 bytes more to continue processing"
    );
    assert_eq!(
        format!("{:?}", error),
        "Invalid { retry_requirement: Some(RetryRequirement(2)) }"
    );
}

#[test]
fn test_invalid_fatal() {
    let error = input!(b"2")
        .read_all::<_, _, Invalid>(|r| r.consume(b"1"))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(format!("{}", error), "invalid input");
    assert_eq!(
        format!("{:?}", error),
        "Invalid { retry_requirement: None }"
    );
}

#[test]
fn test_expected_valid_full() {
    let error = input!(b"123\xC2 ")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err();

    assert_eq!(
        format!("{}", error),
        concat!(
            "error attempting to take str while: expected utf-8 code point\n",
            "> [31 32 33 c2 20]\n",
            "            ^^    \n",
            "additional:\n",
            "  error offset: 3, input length: 5\n",
            "backtrace:\n",
            " 1. `read all`\n",
            " 2. `read` (expected hi)\n",
            " 3. `take str while` (expected utf-8 code point)"
        )
    );
}

#[test]
fn test_expected_valid_full_debug() {
    let error = input!(b"123\xC2 ")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err();

    assert_eq!(
        format!("{:?}", error),
        concat!(
            "\n-- INPUT ERROR ---------------------------------------------\n",
            "error attempting to take str while: expected utf-8 code point\n",
            "> [31 32 33 c2 20]\n",
            "            ^^    \n",
            "additional:\n",
            "  error offset: 3, input length: 5\n",
            "backtrace:\n",
            " 1. `read all`\n",
            " 2. `read` (expected hi)\n",
            " 3. `take str while` (expected utf-8 code point)",
            "\n------------------------------------------------------------\n",
        )
    );
}

#[test]
fn test_expected_length_full() {
    let error = input!(b"123")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.take(5)))
        .unwrap_err();

    assert_eq!(
        format!("{}", error),
        concat!(
            "error attempting to take: found 3 bytes when at least 5 bytes was expected\n",
            "> [31 32 33]\n",
            "   ^^ ^^ ^^ \n",
            "additional:\n",
            "  error offset: 0, input length: 3\n",
            "backtrace:\n",
            " 1. `read all`\n",
            " 2. `read` (expected hi)\n",
            " 3. `take` (expected enough input)",
        )
    );
}

#[test]
fn test_expected_value_full() {
    let error = input!(b"123")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.consume(b"124")))
        .unwrap_err();

    assert_eq!(
        format!("{}", error),
        concat!(
            "error attempting to consume: found a different value to the exact expected\n",
            "expected:\n",
            "> [31 32 34]\n",
            "in:\n",
            "> [31 32 33]\n",
            "   ^^ ^^ ^^ \n",
            "additional:\n",
            "  error offset: 0, input length: 3\n",
            "backtrace:\n",
            " 1. `read all`\n",
            " 2. `read` (expected hi)\n",
            " 3. `consume` (expected exact value)"
        )
    );
}
