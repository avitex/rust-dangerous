#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// Fatal

#[test]
fn test_fatal() {
    let error = input!(b"")
        .read_all::<_, _, Fatal>(|r| r.consume(b"1"))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(format!("{}", error), "invalid input");
    assert_str_eq!(format!("{:?}", error), "Fatal");
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
    assert_str_eq!(
        format!("{}", error),
        "invalid input: needs 1 byte more to continue processing"
    );
    assert_str_eq!(
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
    assert_str_eq!(
        format!("{}", error),
        "invalid input: needs 2 bytes more to continue processing"
    );
    assert_str_eq!(
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
    assert_str_eq!(format!("{}", error), "invalid input");
    assert_str_eq!(
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

impl<'i> WithContext<'i> for ExpectedKind<'i> {
    fn with_input(self, _input: impl Input<'i>) -> Self {
        self
    }

    fn with_context(self, _context: impl Context) -> Self {
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

impl<'i> ToRetryRequirement for ExpectedKind<'i> {
    fn to_retry_requirement(&self) -> Option<RetryRequirement> {
        match self {
            Self::Value(e) => e.to_retry_requirement(),
            Self::Valid(e) => e.to_retry_requirement(),
            Self::Length(e) => e.to_retry_requirement(),
        }
    }
}

fn trigger_expected_valid<E: Error<'static>>() -> E {
    input!(b"hello world\xC2 ")
        .read_all(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err()
}

fn trigger_expected_length<E: Error<'static>>() -> E {
    input!(b"hello world")
        .read_all(|r| r.context("hi", |r| r.take(b"hello world".len() + 2)))
        .unwrap_err()
}

fn trigger_expected_value<E: Error<'static>>() -> E {
    input!(b"hello world")
        .read_all(|r| r.context("hi", |r| r.consume(b"123")))
        .unwrap_err()
}

#[cfg(feature = "full-backtrace")]
fn trigger_expected_value_str<E: Error<'static>>() -> E {
    input!("hello world")
        .read_all(|r| r.context("hi", |r| r.consume("123")))
        .unwrap_err()
}

///////////////////////////////////////////////////////////////////////////////
// Expected valid

#[test]
fn test_expected_valid_variant() {
    let error = match trigger_expected_valid() {
        ExpectedKind::Valid(error) => error,
        _ => unreachable!(),
    };
    assert!(!error.input().is_string());
    assert_eq!(error.input().into_bytes(), b"hello world\xC2 "[..]);
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedValid {
                retry_requirement: None,
                context: CoreContext {
                    span: Span(
                        [c2],
                    ),
                    operation: TakeStrWhile,
                    expected: Valid(
                        "utf-8 code point",
                    ),
                },
                input: Bytes {
                    bound: Start,
                    value: ['h' 'e' 'l' 'l' 'o' 20 'w' 'o' 'r' 'l' 'd' c2 20],
                },
            }
        "#}
    );
}

#[test]
fn test_expected_valid_root() {
    let error: Expected<RootBacktrace> = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take UTF-8 input while a condition remains true: expected utf-8 code point
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64 c2 20]
                                                ^^    
            additional:
              error offset: 11, input length: 13
            backtrace:
              1. `take UTF-8 input while a condition remains true` (expected utf-8 code point)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_expected_valid_full() {
    let error: Expected = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    println!("{}\n", error);
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take UTF-8 input while a condition remains true: expected utf-8 code point
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64 c2 20]
                                                ^^    
            additional:
              error offset: 11, input length: 13
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `take UTF-8 input while a condition remains true` (expected utf-8 code point)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_expected_valid_full_boxed() {
    let error: Expected = trigger_expected_valid();
    let error_boxed: Box<Expected> = trigger_expected_valid();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(format!("{:#?}", error), format!("{:#?}", error_boxed));
}

///////////////////////////////////////////////////////////////////////////////
// Expected length

#[test]
fn test_expected_length_variant() {
    let error = match trigger_expected_length() {
        ExpectedKind::Length(error) => error,
        _ => unreachable!(),
    };
    assert!(!error.input().is_string());
    assert_eq!(error.input().into_bytes(), b"hello world"[..]);
    assert_eq!(error.len(), Length::AtLeast(13));
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_str_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedLength {
                len: AtLeast(
                    13,
                ),
                context: CoreContext {
                    span: Span(
                        "hello world",
                    ),
                    operation: Take,
                    expected: EnoughInputFor(
                        "split",
                    ),
                },
                input: Bytes {
                    bound: Start,
                    value: "hello world",
                },
            }
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_external_error_deep_child() {
    struct DeepExternalError;

    impl<'i> External<'i> for DeepExternalError {
        fn push_backtrace<E>(self, error: E) -> E
        where
            E: WithContext<'i>,
        {
            error.with_context("a").with_context("b").with_context("c")
        }
    }

    let error = read_all_err!("hello world", |r| {
        r.try_external("value", |_| {
            Result::<(usize, ()), DeepExternalError>::Err(DeepExternalError)
        })
    });

    assert!(error.is_fatal());
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to read and expect an external value: expected value
            > "hello world"
               ^^^^^^^^^^^ 
            additional:
              error line: 1, error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `read and expect an external value` (expected value)
                1. `<context>` (expected c)
                2. `<context>` (expected b)
                3. `<context>` (expected a)
        "#}
    );
}

#[test]
fn test_expected_length_root() {
    let error: Expected<RootBacktrace> = trigger_expected_length();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take a length of input: found 11 bytes when at least 13 bytes was expected
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64]
               ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 11
            backtrace:
              1. `take a length of input` (expected enough input for split)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_expected_length_full() {
    let error: Expected = trigger_expected_length();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to take a length of input: found 11 bytes when at least 13 bytes was expected
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64]
               ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ ^^ 
            additional:
              error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `take a length of input` (expected enough input for split)
        "#}
    );
}

///////////////////////////////////////////////////////////////////////////////
// Expected value

#[test]
fn test_expected_value_variant() {
    let error = match trigger_expected_value() {
        ExpectedKind::Value(error) => error,
        _ => unreachable!(),
    };
    assert!(!error.input().is_string());
    assert_eq!(error.input().into_bytes(), b"hello world"[..]);
    assert_eq!(error.expected().as_bytes(), &b"123"[..]);
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{:#?}\n", error),
        indoc! {r#"
            ExpectedValue {
                expected: Bytes(
                    "123",
                ),
                context: CoreContext {
                    span: Span(
                        "hel",
                    ),
                    operation: Consume,
                    expected: ExactValue,
                },
                input: Bytes {
                    bound: Start,
                    value: "hello world",
                },
            }
        "#}
    );
}

#[test]
fn test_expected_value_root() {
    let error: Expected<RootBacktrace> = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    println!("{}", error);
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to consume input: found a different value to the exact expected
            expected:
            > [31 32 33]
            in:
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64]
               ^^ ^^ ^^                         
            additional:
              error offset: 0, input length: 11
            backtrace:
              1. `consume input` (expected exact value)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_expected_value_full() {
    let error: Expected = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to consume input: found a different value to the exact expected
            expected:
            > [31 32 33]
            in:
            > [68 65 6c 6c 6f 20 77 6f 72 6c 64]
               ^^ ^^ ^^                         
            additional:
              error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `consume input` (expected exact value)
        "#}
    );
}

///////////////////////////////////////////////////////////////////////////////
// Other

#[test]
#[cfg(feature = "full-backtrace")]
fn test_error_max_input_len() {
    let error: Expected = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{}\n", error.display().input_max_width(20)),
        indoc! {r#"
            error attempting to consume input: found a different value to the exact expected
            expected:
            > [31 32 33]
            in:
            > [68 65 6c 6c 6f ..]
               ^^ ^^ ^^          
            additional:
              error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `consume input` (expected exact value)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_error_display_str() {
    let error: Expected = trigger_expected_value_str();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{}\n", error),
        indoc! {r#"
            error attempting to consume input: found a different value to the exact expected
            expected:
            > "123"
            in:
            > "hello world"
               ^^^         
            additional:
              error line: 1, error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `consume input` (expected exact value)
        "#}
    );
}

#[test]
#[cfg(feature = "full-backtrace")]
fn test_error_display_str_hint() {
    let error: Expected = trigger_expected_value();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_str_eq!(
        format!("{:#}\n", error),
        indoc! {r#"
            error attempting to consume input: found a different value to the exact expected
            expected:
            > "123"
            in:
            > "hello world"
               ^^^         
            additional:
              error line: 1, error offset: 0, input length: 11
            backtrace:
              1. `read all input`
              2. `<context>` (expected hi)
              3. `consume input` (expected exact value)
        "#}
    );
}

#[test]
fn test_invalid_error_details_span() {
    use dangerous::Input;

    struct BadExternalError;

    impl<'i> External<'i> for BadExternalError {
        fn span(&self) -> Option<Span> {
            Some("not-a-valid-span".into())
        }
    }

    let error = read_all_err!("hello world", |r| {
        r.try_external("value", |_| {
            Result::<(usize, ()), BadExternalError>::Err(BadExternalError)
        })
    });

    assert!(error.is_fatal());
    let error_message = format!("{}", error.display());
    assert_str_eq!(
        // We split the end off as pointers change
        error_message.splitn(2, "additional:").next().unwrap(),
        indoc! {r#"
            error attempting to read and expect an external value: expected value
            note: error span is not within the error input indicating the
                  concrete error being used has a bug. Consider raising an
                  issue with the maintainer!
            input:
            > "hello world"
        "#}
    );
}
