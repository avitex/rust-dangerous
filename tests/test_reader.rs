#[macro_use]
mod common;

use dangerous::Error;

#[test]
fn read_nums() {
    assert_read_all_eq!(&[0x1], |r| r.read_u8(), 1);

    validate_read_num!(i8, le: read_i8_le, be: read_i8_be);
    validate_read_num!(u16, le: read_u16_le, be: read_u16_be);
    validate_read_num!(i16, le: read_i16_le, be: read_i16_be);
    validate_read_num!(u32, le: read_u32_le, be: read_u32_be);
    validate_read_num!(i32, le: read_i32_le, be: read_i32_be);
    validate_read_num!(u64, le: read_u64_le, be: read_u64_be);
    validate_read_num!(i64, le: read_i64_le, be: read_i64_be);
    validate_read_num!(f32, le: read_f32_le, be: read_f32_be);
    validate_read_num!(f64, le: read_f64_le, be: read_f64_be);
}

#[test]
fn read_all() {
    // Valid
    assert_eq!(
        reader!(b"hello")
            .read_all(|r| { r.consume(b"hello") })
            .unwrap(),
        ()
    );
    assert_eq!(
        reader!(b"hello").read_all(|r| { r.take(5) }).unwrap(),
        input!(b"hello")
    );
    // Invalid
    assert_eq!(
        reader!(b"hello")
            .read_all(|r| { r.consume(b"hell") })
            .unwrap_err()
            .can_continue_after(),
        None
    );
    assert_eq!(
        reader!(b"hello")
            .read_all(|r| { r.take(4) })
            .unwrap_err()
            .can_continue_after(),
        None
    );
}

#[test]
fn skip() {
    assert_read_all_eq!(b"hello", |r| { r.skip(5) }, ());
}

#[test]
fn take() {
    assert_read_all_eq!(b"hello", |r| { r.take(5) }, &b"hello"[..]);
}

#[test]
fn take_while() {
    assert_read_all_eq!(
        b"hello!",
        |r| {
            let v = r.take_while(|_, c| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        },
        &b"hello"[..]
    );
}

#[test]
fn try_take_while() {
    // Valid
    assert_read_all_eq!(
        b"hello!",
        |r| {
            let v = r.try_take_while(|_, c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        },
        &b"hello"[..]
    );

    // Invalid
    reader!(b"hello")
        .try_take_while(|i, _| i.reader().consume(b"world").map(|_| true))
        .unwrap_err();
}

#[test]
fn peek() {
    assert_read_all_eq!(
        b"hello",
        |r| {
            let v = r.peek(4, |i| i == b"hell"[..])?;
            r.skip(5)?;
            Ok(v)
        },
        true
    );
}

#[test]
fn try_peek() {
    // Valid
    assert_read_all_eq!(
        b"hello",
        |r| {
            let v = r.try_peek(4, |i| Ok(i == b"hell"[..]))?;
            r.skip(5)?;
            Ok(v)
        },
        true
    );

    // Invalid
    reader!(b"hello")
        .try_peek(4, |i| match i.reader().consume(b"world") {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        })
        .unwrap_err();
}
