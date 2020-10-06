#![allow(clippy::float_cmp)]
#![allow(clippy::unit_cmp)]

#[macro_use]
mod common;
use dangerous::error::{ErrorDetails, ExpectedContext, RetryRequirement, ToRetryRequirement};
use std::any::Any;

#[test]

fn test_read_nums() {
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
fn test_at_end() {
    assert_eq!(
        read_all!(b"hello", |r| {
            r.consume(b"hello")?;
            Ok(r.at_end())
        })
        .unwrap(),
        true
    );
}

#[test]
fn test_context() {
    let err = read_all!(b"hello", |r| { r.context("bob", |r| r.consume(b"world")) }).unwrap_err();
    assert_eq!(err.context_stack().count(), 3);
    err.context_stack().walk(&mut |i, c| {
        // i == 1 is an operation context which cannot be downcast
        if i == 2 {
            let c = Any::downcast_ref::<&'static str>(c.as_any());
            assert_eq!(c, Some(&"bob"));
        }
        if i == 3 {
            let c = Any::downcast_ref::<ExpectedContext>(c.as_any());
            assert!(c.is_some());
        }
        assert!(i != 5);
        true
    });
}

#[test]
fn test_skip() {
    assert_eq!(read_all!(b"hello", |r| { r.skip(5) }).unwrap(), ());
}

#[test]
fn test_skip_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.skip_while(|c| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        5
    );
}

#[test]
fn test_try_skip_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_skip_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        5
    );
}

#[test]
fn test_take() {
    assert_eq!(
        read_all!(b"hello", |r| { r.take(5) }).unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_take_remaining() {
    assert_eq!(
        read_all!(b"hello", |r| { Ok(r.take_remaining()) }).unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_take_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.take_while(|c| c.is_ascii_alphabetic());
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_try_take_while() {
    assert_eq!(
        read_all!(b"hello!", |r| {
            let v = r.try_take_while(|c| Ok(c.is_ascii_alphabetic()))?;
            r.skip(1)?;
            Ok(v)
        })
        .unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_take_consumed() {
    assert_eq!(
        read_all!(b"hello", |r| {
            Ok(r.take_consumed(|r| {
                r.take_remaining();
            }))
        })
        .unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_try_take_consumed() {
    assert_eq!(
        read_all!(b"hello", |r| {
            r.try_take_consumed(|r| r.consume(b"hello"))
        })
        .unwrap(),
        &b"hello"[..]
    );
}

#[test]
fn test_peek() {
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
fn test_peek_eq() {
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
fn test_try_peek() {
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
fn test_consume() {
    // Valid
    assert_eq!(
        read_all!(b"hello", |r| { r.consume(b"hello") }).unwrap(),
        ()
    );
    // Invalid
    assert_eq!(
        read_all!(b"hell", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        RetryRequirement::new(1)
    );
    assert_eq!(
        read_all!(b"abcde", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
    assert_eq!(
        read_all!(b"abc", |r| { r.consume(b"hello") })
            .unwrap_err()
            .to_retry_requirement(),
        None
    );
}

#[test]
fn test_error() {
    use dangerous::{Expected, Invalid, Reader};

    read_all!("hello", |parent: &mut Reader<'_, Expected<'_>>| {
        let mut child = parent.error::<Invalid>();
        assert_eq!(child.consume(b"world"), Err(Invalid::fatal()));
        Ok(())
    })
    .unwrap_err();
}
