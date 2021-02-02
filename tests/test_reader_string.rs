#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// Test debug

#[test]
fn test_string_input_debug() {
    read_all!("hello", |r| {
        assert_eq!(
            format!("{:?}", r),
            r#"Reader { input: String { bound: Start, value: "hello" } }"#
        );
        r.consume("hello")
    })
    .unwrap();
}

#[test]
fn test_string_input_pretty_debug() {
    read_all!("hello", |r| {
        assert_eq!(
            format!("{:#?}\n", r),
            indoc! {r#"
                Reader {
                    input: String {
                        bound: Start,
                        value: "hello",
                    },
                }
            "#}
        );
        r.consume("hello")
    })
    .unwrap();
}

///////////////////////////////////////////////////////////////////////////////
// Test consuming

#[test]
fn test_consume_char() {
    // Valid
    read_all!("1", |r| { r.consume('1') }).unwrap();
    // Invalid
    assert_eq!(
        read_all!("1", |r| { r.consume('2') })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!("", |r| { r.consume('1') })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

#[test]
fn test_consume_char_opt() {
    // Valid
    assert!(read_all!("1", |r| { Ok(r.consume_opt('1')) }).unwrap());
    // Invalid
    assert!(!read_all!("1", |r| {
        let v = r.consume_opt('2');
        r.skip(1)?;
        Ok(v)
    })
    .unwrap());
}

///////////////////////////////////////////////////////////////////////////////
// Test peeking

#[test]
fn test_peek_char() {
    assert_eq!(
        read_all!("hello", |r| {
            let v = r.peek_char()? == 'h';
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}

#[test]
fn test_peek_char_opt() {
    assert_eq!(
        read_all!("hello", |r| {
            let v = r.peek_char_opt().map_or(false, |v| v == 'h');
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}
