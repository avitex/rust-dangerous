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
            r.consume(b'!')?;
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
            r.consume(b'!')?;
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
            r.consume('!')?;
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
            r.consume('!')?;
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
            r.consume(b'!')?;
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
            r.consume(b'!')?;
            Ok(v)
        }),
        b""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// reject: u8

#[test]
fn test_reject_u8() {
    assert_eq!(
        read_all_ok!(b"1111!", |r| {
            let v = r.take_until(b'!')?;
            r.consume(b'!')?;
            Ok(v)
        }),
        b"1111"[..]
    );
}

#[test]
fn test_reject_u8_empty() {
    assert_eq!(
        read_all_ok!(b"!", |r| {
            let v = r.take_until(b'!')?;
            r.consume(b'!')?;
            Ok(v)
        }),
        b""[..]
    );
}

#[test]
fn test_reject_u8_none() {
    let _ = read_all_err!(b"hello", |r| {
        let v = r.take_until(b'!')?;
        r.consume(b'!')?;
        Ok(v)
    });
}

///////////////////////////////////////////////////////////////////////////////
// match: char

#[test]
fn test_match_char() {
    assert_eq!(
        read_all_ok!("1111!", |r| {
            let v = r.take_while('1');
            r.consume('!')?;
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
            r.consume('!')?;
            Ok(v)
        }),
        ""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// reject: char

#[test]
fn test_reject_char() {
    assert_eq!(
        read_all_ok!("1111!", |r| {
            let v = r.take_until('!')?;
            r.consume('!')?;
            Ok(v)
        }),
        "1111"[..]
    );
}

#[test]
fn test_reject_char_empty() {
    assert_eq!(
        read_all_ok!("!", |r| {
            let v = r.take_until('!')?;
            r.consume('!')?;
            Ok(v)
        }),
        ""[..]
    );
}

#[test]
fn test_reject_char_none() {
    let _ = read_all_err!("hello", |r| {
        let v = r.take_until('!')?;
        r.consume('!')?;
        Ok(v)
    });
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
            r.consume('!')?;
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
            r.consume('!')?;
            Ok(v)
        }),
        b""[..]
    );
}

///////////////////////////////////////////////////////////////////////////////
// reject: bytes regex

#[test]
#[cfg(feature = "regex")]
fn test_reject_bytes_regex() {
    assert_eq!(
        read_all_ok!(b"!!!!1234", |r| {
            let regex = regex::bytes::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            r.consume("1234")?;
            Ok(v)
        }),
        b"!!!!"[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_reject_bytes_regex_empty() {
    assert_eq!(
        read_all_ok!(b"1234", |r| {
            let regex = regex::bytes::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            r.consume("1234")?;
            Ok(v)
        }),
        b""[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_reject_bytes_regex_none() {
    assert_eq!(
        read_all_ok!(b"!!!!", |r| {
            let regex = regex::bytes::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            // take_until_opt takes the rest as the pattern is optional.
            r.consume("")?;
            Ok(v)
        }),
        b"!!!!"[..]
    )
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

///////////////////////////////////////////////////////////////////////////////
// reject: bytes regex

#[test]
#[cfg(feature = "regex")]
fn test_reject_string_regex() {
    assert_eq!(
        read_all_ok!("!!!!1234", |r| {
            let regex = regex::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            r.consume("1234")?;
            Ok(v)
        }),
        "!!!!"[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_reject_string_regex_empty() {
    assert_eq!(
        read_all_ok!("1234", |r| {
            let regex = regex::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            r.consume("1234")?;
            Ok(v)
        }),
        ""[..]
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_reject_string_regex_none() {
    assert_eq!(
        read_all_ok!("!!!!", |r| {
            let regex = regex::Regex::new("\\d+").unwrap();
            let v = r.take_until_opt(&regex);
            // take_until_opt takes the rest as the pattern is optional.
            r.consume("")?;
            Ok(v)
        }),
        "!!!!"[..]
    )
}
