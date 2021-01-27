#[macro_use]
mod common;

use common::*;
use std::fmt;

///////////////////////////////////////////////////////////////////////////////
// Fatal

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

///////////////////////////////////////////////////////////////////////////////
// Invalid

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

///////////////////////////////////////////////////////////////////////////////
// Expected support

enum ExpectedKind<'i> {
    Value(ExpectedValue<'i>),
    Valid(ExpectedValid<'i>),
    Length(ExpectedLength<'i>),
}

impl<'i> fmt::Debug for ExpectedKind<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(e) => e.fmt(f),
            Self::Valid(e) => e.fmt(f),
            Self::Length(e) => e.fmt(f),
        }
    }
}

impl<'i> WithContext<'i> for ExpectedKind<'i> {
    fn with_context(self, _input: impl Input<'i>, _context: impl Context) -> Self {
        self
    }
}

impl<'i> From<ExpectedValid<'i>> for ExpectedKind<'i> {
    fn from(err: ExpectedValid<'i>) -> Self {
        Self::Valid(err)
    }
}

impl<'i> From<ExpectedValue<'i>> for ExpectedKind<'i> {
    fn from(err: ExpectedValue<'i>) -> Self {
        Self::Value(err)
    }
}

impl<'i> From<ExpectedLength<'i>> for ExpectedKind<'i> {
    fn from(err: ExpectedLength<'i>) -> Self {
        Self::Length(err)
    }
}

fn trigger_expected_valid<E: Error<'static>>() -> E {
    input!(b"123\xC2 ")
        .read_all(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err()
}

fn trigger_expected_length<E: Error<'static>>() -> E {
    input!(b"123")
        .read_all(|r| r.context("hi", |r| r.take(5)))
        .unwrap_err()
}

fn trigger_expected_value<E: Error<'static>>() -> E {
    input!(b"123")
        .read_all(|r| r.context("hi", |r| r.consume(b"124")))
        .unwrap_err()
}

///////////////////////////////////////////////////////////////////////////////
// Expected valid

#[test]
fn test_expected_valid_variant() {
    let error: ExpectedKind = trigger_expected_valid();

    assert_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedValid {
                input: Bytes {
                    bound: Start,
                    bytes: ['1' '2' '3' c2 20],
                },
                span: Bytes {
                    bound: Start,
                    bytes: [c2],
                },
                context: ExpectedContext {
                    operation: "take str while",
                    expected: "utf-8 code point",
                },
                retry_requirement: None,
            }
        "#}
    );
}

#[test]
fn test_expected_valid_root() {
    let error: Expected<RootContextStack> = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take str while: expected utf-8 code point
            > [31 32 33 c2 20]
                        ^^    
            additional:
              error offset: 3, input length: 5
            backtrace:
              1. `take str while` (expected utf-8 code point)
        "#}
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_valid_full() {
    let error: Expected = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    println!("{}\n", error);
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take str while: expected utf-8 code point
            > [31 32 33 c2 20]
                        ^^    
            additional:
              error offset: 3, input length: 5
            backtrace:
              1. `read all`
              2. `read` (expected hi)
              3. `take str while` (expected utf-8 code point)
        "#}
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_valid_full_boxed() {
    let error: Expected = trigger_expected_valid();
    let error_boxed: Box<Expected> = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(format!("{:#?}", error), format!("{:#?}", error_boxed),);
}

///////////////////////////////////////////////////////////////////////////////
// Expected length

#[test]
fn test_expected_length_variant() {
    let error: ExpectedKind = trigger_expected_length();

    assert_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedLength {
                min: 5,
                max: None,
                input: Bytes {
                    bound: Start,
                    bytes: "123",
                },
                span: Bytes {
                    bound: Start,
                    bytes: "123",
                },
                context: ExpectedContext {
                    operation: "take",
                    expected: "enough input",
                },
            }
        "#}
    );
}

#[test]
fn test_expected_length_root() {
    let error: Expected<RootContextStack> = trigger_expected_length();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take: found 3 bytes when at least 5 bytes was expected
            > [31 32 33]
               ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 3
            backtrace:
              1. `take` (expected enough input)
        "#}
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_length_full() {
    let error: Expected = trigger_expected_length();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take: found 3 bytes when at least 5 bytes was expected
            > [31 32 33]
               ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 3
            backtrace:
              1. `read all`
              2. `read` (expected hi)
              3. `take` (expected enough input)
        "#},
    );
}

///////////////////////////////////////////////////////////////////////////////
// Expected value

#[test]
fn test_expected_value() {
    let error: ExpectedKind = trigger_expected_value();

    assert_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedValue {
                input: Bytes {
                    bound: Start,
                    bytes: "123",
                },
                actual: Bytes {
                    bound: Start,
                    bytes: "123",
                },
                expected: Bytes(
                    "124",
                ),
                context: ExpectedContext {
                    operation: "consume",
                    expected: "exact value",
                },
            }
        "#}
    );
}

#[test]
fn test_expected_value_root() {
    let error: Expected<RootContextStack> = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    println!("{}", error);
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to consume: found a different value to the exact expected
            expected:
            > [31 32 34]
            in:
            > [31 32 33]
               ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 3
            backtrace:
              1. `consume` (expected exact value)
        "#}
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_value_full() {
    let error: Expected = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to consume: found a different value to the exact expected
            expected:
            > [31 32 34]
            in:
            > [31 32 33]
               ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 3
            backtrace:
              1. `read all`
              2. `read` (expected hi)
              3. `consume` (expected exact value)
        "#},
    );
}

///////////////////////////////////////////////////////////////////////////////
// Other

#[test]
fn test_invalid_error_details_span() {
    use dangerous::display::{ErrorDisplay, Write};
    use dangerous::error::{
        ContextStack, ContextStackBuilder, Details, ExpectedValid, RootContextStack,
    };
    use dangerous::{Input, MaybeString};

    struct MyError(RootContextStack);

    impl<'i> From<ExpectedValid<'i>> for MyError {
        fn from(err: ExpectedValid<'i>) -> Self {
            Self(RootContextStack::from_root(err.context()))
        }
    }

    impl<'i> Details<'i> for MyError {
        fn input(&self) -> MaybeString<'i> {
            input!(b"something").into_maybe_string()
        }
        fn span(&self) -> Bytes<'i> {
            input!(b"not-a-proper-span")
        }
        fn expected(&self) -> Option<Value<'_>> {
            None
        }
        fn description(&self, w: &mut dyn Write) -> fmt::Result {
            w.write_str("test")
        }
        fn context_stack(&self) -> &dyn ContextStack {
            &self.0
        }
    }

    let error = input!(b"\xC2 ").to_dangerous_str::<MyError>().unwrap_err();

    assert_eq!(
        // We split the end off as pointers change
        format!("{}", ErrorDisplay::new(&error))
            .splitn(2, "additional:")
            .next()
            .unwrap(),
        indoc! {"
            error attempting to convert input to str: test
            note: error span is not within the error input indicating the
                  concrete error being used has a bug. Consider raising an
                  issue with the maintainer!
            span:
            > [6e 6f 74 2d 61 2d 70 72 6f 70 65 72 2d 73 70 61 6e]
            input:
            > [73 6f 6d 65 74 68 69 6e 67]
        "}
    );
}
