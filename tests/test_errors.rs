use core::fmt;

#[macro_use]
mod common;

use dangerous::error::{
    Expected, Fatal, Invalid, RetryRequirement, RootContextStack, ToRetryRequirement,
};

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
#[cfg(feature = "full-context")]
fn test_expected_valid_full() {
    let error = input!(b"123\xC2 ")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
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
fn test_expected_valid_root() {
    let error = input!(b"123\xC2 ")
        .read_all::<_, _, Expected<RootContextStack>>(|r| {
            r.context("hi", |r| r.take_str_while(|_| true))
        })
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(
        format!("{}", error),
        concat!(
            "error attempting to take str while: expected utf-8 code point\n",
            "> [31 32 33 c2 20]\n",
            "            ^^    \n",
            "additional:\n",
            "  error offset: 3, input length: 5\n",
            "backtrace:\n",
            " 1. `take str while` (expected utf-8 code point)"
        )
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_valid_full_debug_str_boxed() {
    let error = input!(b"123\n\xC2 ")
        .read_all::<_, _, Box<Expected>>(|r| r.context("hi", |r| r.take_str_while(|_| true)))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
    assert_eq!(
        format!("{:#?}", error),
        concat!(
            "\n-- INPUT ERROR ---------------------------------------------\n",
            "error attempting to take str while: expected utf-8 code point\n",
            "> ['1' '2' '3' '\\n' c2 20]\n",
            "                    ^^    \n",
            "additional:\n",
            "  error line: 2, error offset: 4, input length: 6\n",
            "backtrace:\n",
            " 1. `read all`\n",
            " 2. `read` (expected hi)\n",
            " 3. `take str while` (expected utf-8 code point)\n",
            "------------------------------------------------------------\n"
        )
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_length_full() {
    let error = input!(b"123")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.take(5)))
        .unwrap_err();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
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
fn test_expected_length_root() {
    let error = input!(b"123")
        .read_all::<_, _, Expected<RootContextStack>>(|r| r.context("hi", |r| r.take(5)))
        .unwrap_err();

    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(2));
    assert_eq!(
        format!("{}", error),
        concat!(
            "error attempting to take: found 3 bytes when at least 5 bytes was expected\n",
            "> [31 32 33]\n",
            "   ^^ ^^ ^^ \n",
            "additional:\n",
            "  error offset: 0, input length: 3\n",
            "backtrace:\n",
            " 1. `take` (expected enough input)",
        )
    );
}

#[test]
#[cfg(feature = "full-context")]
fn test_expected_value_full() {
    let error = input!(b"123")
        .read_all::<_, _, Expected>(|r| r.context("hi", |r| r.consume(b"124")))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
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

#[test]
fn test_expected_value_root() {
    let error = input!(b"123")
        .read_all::<_, _, Expected<RootContextStack>>(|r| r.context("hi", |r| r.consume(b"124")))
        .unwrap_err();

    assert!(error.is_fatal());
    assert_eq!(error.to_retry_requirement(), None);
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
            " 1. `consume` (expected exact value)"
        )
    );
}

#[test]
fn test_invalid_error_details_span() {
    use dangerous::display::{ErrorDisplay, Write};
    use dangerous::error::{
        ContextStack, ContextStackBuilder, Details, ExpectedValid, RootContextStack,
    };
    use dangerous::Input;

    struct MyError(RootContextStack);

    impl<'i> From<ExpectedValid<'i>> for MyError {
        fn from(err: ExpectedValid<'i>) -> Self {
            Self(RootContextStack::from_root(err.context()))
        }
    }

    impl<'i> Details<'i> for MyError {
        fn input(&self) -> Input<'i> {
            input!(b"something")
        }
        fn span(&self) -> Input<'i> {
            input!(b"not-a-proper-span")
        }
        fn expected(&self) -> Option<Input<'_>> {
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
        format!("{}", ErrorDisplay::new(&error))
            .splitn(2, "additional:")
            .next()
            .unwrap(),
        concat!(
            "error attempting to convert input to str: test\n",
            "note: error span is not within the error input indicating the\n",
            "      concrete error being used has a bug. Consider raising an\n",
            "      issue with the maintainer!\n",
            "span:\n",
            "> [6e 6f 74 2d 61 2d 70 72 6f 70 65 72 2d 73 70 61 6e]\n",
            "input:\n> [73 6f 6d 65 74 68 69 6e 67]\n",
        )
    );
}
