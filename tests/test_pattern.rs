#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////
// match: bytes function

#[test]
fn test_match_bytes_fn() {
    assert_eq!(
        read_all_ok!(b"hello!", |r| {
            let v = r.take_while(|c: u8| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        }),
        b"hello"[..]
    );
}

#[test]
fn test_match_bytes_fn_none() {
    assert_eq!(
        read_all_ok!(b"!", |r| {
            let v = r.take_while(|c: u8| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        }),
        b""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// match: string function

#[test]
fn test_match_string_fn() {
    assert_eq!(
        read_all_ok!("hello!", |r| {
            let v = r.take_while(|c: char| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        }),
        "hello"[..]
    );
}

#[test]
fn test_match_string_fn_none() {
    assert_eq!(
        read_all_ok!("!", |r| {
            let v = r.take_while(|c: char| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        }),
        ""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// match: u8

#[test]
fn test_match_u8() {
    assert_eq!(
        read_all_ok!(b"1111!", |r| {
            let v = r.take_while(b'1');
            r.skip(1)?;
            Ok(v)
        }),
        b"1111"[..]
    );
}

#[test]
fn test_match_u8_none() {
    assert_eq!(
        read_all_ok!(b"!", |r| {
            let v = r.take_while(b'1');
            r.skip(1)?;
            Ok(v)
        }),
        b""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// match: char

#[test]
fn test_match_char() {
    assert_eq!(
        read_all_ok!("1111!", |r| {
            let v = r.take_while('1');
            r.skip(1)?;
            Ok(v)
        }),
        "1111"[..]
    );
}

#[test]
fn test_match_char_none() {
    assert_eq!(
        read_all_ok!("!", |r| {
            let v = r.take_while('1');
            r.skip(1)?;
            Ok(v)
        }),
        ""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// match: bytes regex

#[test]
#[cfg(feature = "regex")]
fn test_match_bytes_regex() {
    assert_eq!(
        read_all_ok!(b"1234!", |r| {
            let regex = regex::bytes::Regex::new("\\d+").unwrap();
            let v = r.take_while(&regex);
            r.skip(1)?;
            Ok(v)
        }),
        b"1234"[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_match_bytes_regex_none() {
    assert_eq!(
        read_all_ok!(b"!", |r| {
            let regex = regex::bytes::Regex::new("\\d+").unwrap();
            let v = r.take_while(&regex);
            r.skip(1)?;
            Ok(v)
        }),
        b""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// match: string regex

#[test]
#[cfg(feature = "regex")]
fn test_match_string_regex() {
    assert_eq!(
        read_all_ok!("1234!", |r| {
            let regex = regex::Regex::new("\\d+").unwrap();
            let v = r.take_while(&regex);
            r.skip(1)?;
            Ok(v)
        }),
        "1234"[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_match_string_regex_none() {
    assert_eq!(
        read_all_ok!("!", |r| {
            let regex = regex::Regex::new("\\d+").unwrap();
            let v = r.take_while(&regex);
            r.skip(1)?;
            Ok(v)
        }),
        ""[..]
    );
}
