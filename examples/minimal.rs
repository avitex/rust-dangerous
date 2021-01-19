use core::fmt::Debug;

use dangerous::{Error, Fatal, Input, Invalid};

fn expected<'i, E>(bytes: &'i [u8]) -> Result<(), E>
where
    E: Error<'i>,
{
    dangerous::input(bytes).read_all(|r| {
        r.context("foo", |r| {
            r.context("bar", |r| {
                r.context("hello", |r| r.context("world", |r| r.consume(b"o")))
            })
        })
    })
}

#[inline(never)]
fn invalid_ok() -> Result<(), Invalid> {
    expected(b"o")
}

#[inline(never)]
fn invalid_err() -> Result<(), Invalid> {
    expected(b"e")
}

#[inline(never)]
fn fatal_ok() -> Result<(), Fatal> {
    expected(b"o")
}

#[inline(never)]
fn fatal_err() -> Result<(), Fatal> {
    expected(b"e")
}

fn main() {
    handle(fatal_ok());
    handle(fatal_err());
    handle(invalid_ok());
    handle(invalid_err());
}

fn handle<E: Debug>(result: Result<(), E>) {
    dbg!(&result);
}
