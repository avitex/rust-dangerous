#![allow(clippy::unit_cmp)]

#[macro_use]
mod common;

use common::*;
use std::any::Any;

///////////////////////////////////////////////////////////////////////////////
// Reader::at_end

#[test]
fn test_at_end_true() {
    assert!(read_all_ok!(b"hello", |r| {
        r.consume(b"hello")?;
        Ok(r.at_end())
    }));
}

#[test]
fn test_at_end_false() {
    assert_eq!(
        read_partial_ok!(b"hello", |r| {
            r.consume(b"hell")?;
            Ok(r.at_end())
        }),
        (false, input!(b"o"))
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::context

#[test]
fn test_context() {
    let err = read_all_err!(b"hello", |r| { r.context("bob", |r| r.consume(b"world")) });
    #[cfg(feature = "full-backtrace")]
    assert_eq!(err.backtrace().count(), 3);
    #[cfg(not(feature = "full-backtrace"))]
    assert_eq!(err.backtrace().count(), 1);
    err.backtrace().walk(&mut |i, c| {
        let c = <dyn Any>::downcast_ref::<CoreOperation>(c.operation().as_any());
        assert!(c.is_some());
        assert!(i != 5);
        true
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::skip

#[test]
fn test_skip() {
    assert_eq!(read_all_ok!(b"hello", |r| { r.skip(5) }), ());
}

///////////////////////////////////////////////////////////////////////////////
// Reader::skip_opt

#[test]
fn test_skip_opt_true() {
    assert_eq!(
        read_infallible!(b"hello", |r| { r.skip_opt(5) }),
        (true, input!(b""))
    );
}

#[test]
fn test_skip_opt_false() {
    assert_eq!(
        read_infallible!(b"hello", |r| { r.skip_opt(6) }),
        (false, input!(b"hello"))
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::skip_while

#[test]
fn test_skip_while() {
    read_all_ok!(b"hello!", |r| {
        r.skip_while(|c: u8| c.is_ascii_alphabetic());
        r.skip(1)
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_skip_while

#[test]
fn test_try_skip_while() {
    read_all_ok!(b"hello!", |r| {
        r.try_skip_while(|c| Ok(c.is_ascii_alphabetic()))?;
        r.skip(1)
    })
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take

#[test]
fn test_take() {
    assert_eq!(read_all_ok!(b"hello", |r| { r.take(5) }), b"hello"[..]);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take_opt

#[test]
fn test_take_opt_true() {
    assert_eq!(
        read_infallible!(b"hello", |r| { r.take_opt(5) }),
        (Some(input!(b"hello"[..])), input!(b""))
    );
}

#[test]
fn test_take_opt_false() {
    assert_eq!(
        read_infallible!(b"hello", |r| { r.take_opt(6) }),
        (None, input!(b"hello"))
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take_remaining

#[test]
fn test_take_remaining() {
    let remaining = read_all_ok!(b"hello", |r| { Ok(r.take_remaining()) });
    assert_eq!(remaining, b"hello"[..]);
    assert_eq!(remaining.bound(), Bound::Start);
}

#[test]
fn test_take_remaining_second() {
    let remaining = read_all_ok!(b"hello", |r| {
        let _ = r.take_remaining();
        Ok(r.take_remaining())
    });
    assert_eq!(remaining, b""[..]);
    assert_eq!(remaining.bound(), Bound::None);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take_while

#[test]
fn test_take_while_all() {
    let input = read_all_ok!(b"hello", |r| {
        Ok(r.take_while(|c: u8| c.is_ascii_alphabetic()))
    });
    assert_eq!(input, b"hello"[..]);
    assert_eq!(input.bound(), Bound::Start);
}

#[test]
fn test_take_while_partial() {
    let input = read_all_ok!(b"hello!", |r| {
        let input = r.take_while(|c: u8| c.is_ascii_alphabetic());
        r.skip(1)?;
        Ok(input)
    });
    assert_eq!(input, b"hello"[..]);
    assert_eq!(input.bound(), Bound::StartEnd);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_take_while

#[test]
fn test_try_take_while_all() {
    let input = read_all_ok!(b"hello", |r| {
        r.try_take_while(|c| Ok(c.is_ascii_alphabetic()))
    });
    assert_eq!(input, b"hello"[..]);
    assert_eq!(input.bound(), Bound::Start);
}

#[test]
fn test_try_take_while_partial() {
    let input = read_all_ok!(b"hello!", |r| {
        let v = r.try_take_while(|c| Ok(c.is_ascii_alphabetic()))?;
        r.skip(1)?;
        Ok(v)
    });
    assert_eq!(input, b"hello"[..]);
    assert_eq!(input.bound(), Bound::StartEnd);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::take_consumed

#[test]
fn test_take_consumed_all_unbound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        Ok(r.take_consumed(|r| {
            let _ = r.take_remaining();
        }))
    });
    assert_eq!(consumed, b"hello"[..]);
    assert_eq!(consumed.bound(), Bound::Start);
}

#[test]
fn test_take_consumed_all_bound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        Ok(r.take_consumed(|r| {
            let _ = r.consume(b"hello");
        }))
    });
    assert_eq!(consumed, b"hello"[..]);
    assert_eq!(consumed.bound(), Bound::StartEnd);
}

#[test]
fn test_take_consumed_partial_bound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        let consumed = r.take_consumed(|r| {
            let _ = r.take(2);
            let _ = r.take(1);
        });
        r.consume("lo")?;
        Ok(consumed)
    });
    assert_eq!(consumed, b"hel"[..]);
    assert_eq!(consumed.bound(), Bound::StartEnd);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_take_consumed

#[test]
fn test_try_take_consumed_all_unbound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        r.try_take_consumed(|r| {
            let _ = r.take_remaining();
            Ok(())
        })
    });
    assert_eq!(consumed, b"hello"[..]);
    assert_eq!(consumed.bound(), Bound::Start);
}

#[test]
fn test_try_take_consumed_all_bound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        r.try_take_consumed(|r| {
            r.consume(b"hello")?;
            Ok(())
        })
    });
    assert_eq!(consumed, b"hello"[..]);
    assert_eq!(consumed.bound(), Bound::StartEnd);
}

#[test]
fn test_try_take_consumed_partial_bound() {
    let (_, consumed) = read_all_ok!(b"hello", |r| {
        let consumed = r.try_take_consumed(|r| {
            let _ = r.take(2)?;
            let _ = r.take(1)?;
            Ok(())
        })?;
        r.consume("lo")?;
        Ok(consumed)
    });
    assert_eq!(consumed, b"hel"[..]);
    assert_eq!(consumed.bound(), Bound::StartEnd);
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek

#[test]
fn test_peek_enough() {
    assert!(read_all_ok!(b"hello", |r| {
        let v = *r.peek(4)? == b"hell"[..];
        r.skip(5)?;
        Ok(v)
    }));
}

#[test]
fn test_peek_too_much() {
    let _ = read_all_err!(b"hello", |r| {
        let _ = r.peek(6)?;
        r.skip(5)
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_opt

#[test]
fn test_peek_opt_enough() {
    assert!(read_all_ok!(b"hello", |r| {
        let v = r.peek_opt(4).map_or(false, |v| *v == b"hell"[..]);
        r.skip(5)?;
        Ok(v)
    }));
}

#[test]
fn test_peek_opt_too_much() {
    assert!(!read_all_ok!(b"hello", |r| {
        let v = r.peek_opt(10).map_or(false, |v| *v == b"hell"[..]);
        r.skip(5)?;
        Ok(v)
    }));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::peek_eq

#[test]
fn test_peek_eq_exact_same() {
    let _ = read_partial_ok!(b"helloworld", |r| {
        assert!(r.peek_eq(b"helloworld"));
        Ok(())
    });
}

#[test]
fn test_peek_eq_same_value_different_len() {
    let _ = read_partial_ok!(b"helloworld", |r| {
        assert!(r.peek_eq(b"hello"));
        Ok(())
    });
}

#[test]
fn test_peek_eq_shorter_and_different_value() {
    let _ = read_partial_ok!(b"helloworld", |r| {
        assert!(!r.peek_eq(b"no"));
        Ok(())
    });
}

#[test]
fn test_peek_eq_longer() {
    let _ = read_partial_ok!(b"helloworld", |r| {
        assert!(!r.peek_eq(b"helloworld!"));
        Ok(())
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume

#[test]
fn test_consume_exact_same() {
    assert_eq!(read_all_ok!(b"hello", |r| { r.consume(b"hello") }), ());
}

#[test]
fn test_consume_same_value_different_len() {
    assert_eq!(
        read_all_err!(b"hell", |r| { r.consume(b"hello") }).to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

#[test]
fn test_consume_different_len_and_value() {
    assert_eq!(
        read_all_err!(b"abc", |r| { r.consume(b"hello") }).to_retry_requirement(),
        None
    );
}

#[test]
fn test_consume_same_len_different_value() {
    assert_eq!(
        read_all_err!(b"abcde", |r| { r.consume(b"hello") }).to_retry_requirement(),
        None
    );
}

///////////////////////////////////////////////////////////////////////////////
// Reader::consume_opt

#[test]
fn test_consume_opt_true() {
    assert!(read_all_ok!(b"hello", |r| { Ok(r.consume_opt(b"hello")) }));
}

#[test]
fn test_consume_opt_false() {
    assert!(!read_all_ok!(b"abc", |r| {
        let v = r.consume_opt(b"hello");
        r.skip(3)?;
        Ok(v)
    }));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::verify

#[test]
fn test_verify_true() {
    read_all_ok!(b"1", |r| { r.verify("value", |r| r.consume_opt(b"1")) });
}

#[test]
fn test_verify_false() {
    let _ = read_all_err!(b"1", |r| { r.verify("value", |r| r.consume_opt(b"2")) });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_verify

#[test]
fn test_try_verify_ok_true() {
    read_all_ok!(b"1", |r| {
        r.try_verify("value", |r| Ok(r.consume_opt(b"1")))
    })
}

#[test]
fn test_try_verify_ok_false() {
    let _ = read_all_err!(b"1", |r| {
        r.try_verify("value", |r| Ok(r.consume_opt(b"2")))
    });
}

#[test]
fn test_try_verify_err() {
    let _ = read_all_err!(b"1", |r| {
        r.try_verify("value", |r| r.consume(b"2").map(|()| true))
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::expect

#[test]
fn test_expect_some() {
    read_all_ok!(b"1", |r| {
        r.expect("value", |r| Some(r.consume_opt(b"1")))
    });
}

#[test]
fn test_expect_none() {
    let _ = read_all_err!(b"", |r| { r.expect("value", |_| Option::<()>::None) });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_expect

#[test]
fn test_try_expect_some() {
    read_all_ok!(b"", |r| { r.try_expect("value", |_| Ok(Some(()))) });
}

#[test]
fn test_try_expect_none() {
    let _ = read_all_err!(b"", |r| {
        r.try_expect("value", |_| Ok(Option::<()>::None))
    });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::try_external

struct ExternalError(Option<RetryRequirement>);

impl<'i> External<'i> for ExternalError {
    fn retry_requirement(&self) -> Option<RetryRequirement> {
        self.0
    }
}

#[test]
fn try_external_ok() {
    read_all_ok!(b"", |r| {
        r.try_external("value", |i| Result::<_, ExternalError>::Ok((i.len(), ())))
    });
}

#[test]
fn try_external_unconsumed() {
    let _ = read_all_err!(b"abc", |r| {
        r.try_external("value", |i| {
            Result::<_, ExternalError>::Ok((i.len() - 1, ()))
        })
    });
}

#[test]
fn try_external_read_too_much() {
    let _ = read_all_err!(b"abc", |r| {
        r.try_external("value", |i| {
            Result::<_, ExternalError>::Ok((i.len() + 1, ()))
        })
    });
}

#[test]
fn try_external_read_invalid_boundary() {
    assert_eq!('♥'.len_utf8(), 3);
    let _ = read_all_err!("♥", |r| {
        r.try_external("value", |i| {
            Result::<_, ExternalError>::Ok((i.byte_len() - 1, ()))
        })
    });
}

#[test]
fn try_external_err() {
    let error = read_all_err!(b"", |r| {
        r.try_external("value", |_| {
            Result::<(usize, ()), ExternalError>::Err(ExternalError(RetryRequirement::new(0)))
        })
    });
    assert!(error.is_fatal());
}

#[test]
fn try_external_err_retry() {
    let error = read_all_err!(b"", |r| {
        r.try_external("value", |_| {
            Result::<(usize, ()), ExternalError>::Err(ExternalError(RetryRequirement::new(1)))
        })
    });
    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(1));
}

///////////////////////////////////////////////////////////////////////////////
// Reader::recover

#[test]
fn test_recover() {
    read_all_ok!(b"", |r| {
        r.recover(|r| r.take(1));
        Ok(())
    })
}

///////////////////////////////////////////////////////////////////////////////
// Reader::recover_if

#[test]
fn test_recover_if_ok() {
    read_all_ok!(b"1", |r| { r.recover_if(|r| { r.take(1) }, |_| false) });
}

#[test]
fn test_recover_if_true() {
    read_all_ok!(b"", |r| { r.recover_if(|r| { r.take(1) }, |_| true) });
}

#[test]
fn test_recover_if_false() {
    let _ = read_all_err!(b"", |r| { r.recover_if(|r| { r.take(1) }, |_| false) });
}

///////////////////////////////////////////////////////////////////////////////
// Reader::error

#[test]
fn test_error() {
    let _: Result<_, Expected> = read_all!(b"", |r| {
        let result = r.error(|r: &mut BytesReader<Fatal>| {
            // Normally this would retryable but we are using the fatal error
            r.take(1)
        });
        assert_eq!(result, Err(Fatal));
        Ok(())
    });
}
