#[macro_use]
mod common;

use dangerous::ErrorDetails;

#[test]
fn read_nums() {
    assert_eq!(read_all!(&[0x1], |r| r.read_u8()).unwrap(), 1);

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
        read_all!(b"hello", |r| { r.consume(b"hello") }).unwrap(),
        ()
    );
    assert_eq!(
        read_all!(b"hello", |r| { r.take(5) }).unwrap(),
        input!(b"hello")
    );
    // Invalid
    assert_eq!(
        read_all!(b"hello", |r| { r.consume(b"hell") })
            .unwrap_err()
            .retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"hello", |r| { r.take(4) })
            .unwrap_err()
            .retry_requirement(),
        None
    );
}

#[test]
fn skip() {
    assert_eq!(read_all!(b"hello", |r| { r.skip(5) }).unwrap(), ());
}

#[test]
fn take() {
    assert_eq!(
        read_all!(b"hello", |r| { r.take(5) }).unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn take_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.take_while(|_, c| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn try_take_while() {
    // Valid
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_take_while(|_, c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        &b"hello"[..]
    );

    // Invalid
    read_all!(b"hello", |r| {
        r.try_take_while(|i, _| i.read_all(|r| r.consume(b"world")).map(|_| true))
    })
    .unwrap_err();
}

#[test]
fn peek() {
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.peek(4, |i| i == b"hell"[..])?;
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );
}

#[test]
fn peek_eq() {
    read_partial!(b"helloworld", |r| {
        assert!(r.peek_eq(b"helloworld"));
        assert!(r.peek_eq(b"hello"));
        assert!(!r.peek_eq(b"no"));
        assert!(!r.peek_eq(b"helloworld!"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn try_peek() {
    // Valid
    assert_eq!(
        read_all!(b"hello", |r| {
            let v = r.try_peek(4, |i| Ok(i == b"hell"[..]))?;
            r.skip(5)?;
            Ok(v)
        })
        .unwrap(),
        true
    );

    // Invalid
    read_all!(b"hello", |r| {
        r.try_peek(4, |i| i.read_all(|r| r.consume(b"world")).map(drop))
    })
    .unwrap_err();
}

#[test]
fn error() {
    use dangerous::{Expected, Invalid, Reader};

    read_all!("hello", |parent: &mut Reader<'_, Expected<'_>>| {
        let mut child = parent.error::<Invalid>();
        assert_eq!(child.consume(b"world"), Err(Invalid::default()));
        Ok(())
    })
    .unwrap_err();
}
