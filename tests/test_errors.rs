#[macro_use]
mod common;

use dangerous::error::RetryRequirement;
use dangerous::{Fatal, Invalid, ToRetryRequirement};

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
fn test_invalid_retry() {
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
