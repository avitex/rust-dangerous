#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// Test debug

#[test]
fn test_string_input_debug() {
    read_all_ok!("hello", |r| {
        assert_eq!(
            format!("{:?}", r),
            r#"Reader { input: String { bound: Start, value: "hello" } }"#
        );
        r.consume("hello")
    })
}

#[test]
fn test_string_input_pretty_debug() {
    read_all_ok!("hello", |r| {
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
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume (char)

#[test]
fn test_consume_char_exact_same() {
    read_all_ok!("1", |r| { r.consume('1') });
}

#[test]
fn test_consume_char_same_len_different_value() {
    assert_eq!(
        read_all_err!("1", |r| { r.consume('2') }).to_retry_requirement(),
        None
    );
}

#[test]
fn test_consume_char_different_len_and_value() {
    assert_eq!(
        read_all_err!("", |r| { r.consume('1') }).to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume_opt (char)

#[test]
fn test_consume_opt_char_true() {
    assert!(read_all_ok!("1", |r| { Ok(r.consume_opt('1')) }));
}

#[test]
fn test_consume_opt_char_false() {
    assert!(!read_all_ok!("1", |r| {
        let v = r.consume_opt('2');
        r.skip(1)?;
        Ok(v)
    }));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_read

#[test]
fn test_peek_read() {
    assert!(read_all_ok!("hello", |r| {
        let v = r.peek_read()? == 'h';
        r.skip(5)?;
        Ok(v)
    }));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_read_opt

#[test]
fn test_peek_read_opt() {
    assert!(read_all_ok!("hello", |r| {
        let v = r.peek_read_opt().map_or(false, |v| v == 'h');
        r.skip(5)?;
        Ok(v)
    }));
}
