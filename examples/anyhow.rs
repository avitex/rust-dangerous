use anyhow::Context;
use dangerous::{BytesReader, Error, Expected, Input};

fn main() {
    dangerous::input(b"hello")
        .read_all(parse::<Expected<'_>>)
        .map_err(|err| anyhow::Error::msg(err.to_string()))
        .context("my anyhow context 1")
        .context("my anyhow context 2")
        .unwrap();
}

fn parse<'i, E>(r: &mut BytesReader<'i, E>) -> Result<(), E>
where
    E: Error<'i>,
{
    r.context("my value", |r| r.consume(b"world"))
}
