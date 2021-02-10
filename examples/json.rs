//! This example demonstrates a simple JSON parser (doesn't support escaping
//! fully).
//!
//! ```
//! echo '{ "hello": "bob" }' | cargo run --example json
//! ```
use std::io::{self, Read};

use dangerous::{BytesReader, Error, Expected, Input};

fn main() {
    let mut input_data = Vec::new();
    io::stdin()
        .read_to_end(&mut input_data)
        .expect("read input");
    let input = dangerous::input(input_data.as_slice());
    match input.read_all::<_, _, Box<Expected>>(read_value) {
        Ok(json) => println!("{:#?}", json),
        Err(e) => eprintln!("{:#}", e),
    }
}

#[derive(Debug)]
enum Value<'a> {
    Null,
    Bool(bool),
    Str(&'a str),
    Number(f64),
    Array(Vec<Value<'a>>),
    Object(Vec<(&'a str, Value<'a>)>),
}

fn read_value<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Value<'i>, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    let value = r.try_expect("json value", |r| {
        let value = match r.peek_u8()? {
            b'"' => Value::Str(read_str(r)?),
            b'{' => Value::Object(read_map(r)?),
            b'[' => Value::Array(read_arr(r)?),
            b'n' => {
                read_null(r)?;
                Value::Null
            }
            b't' | b'f' => Value::Bool(read_bool(r)?),
            c if c.is_ascii_digit() => Value::Number(read_num(r)?),
            _ => return Ok(None),
        };
        Ok(Some(value))
    })?;
    skip_whitespace(r);
    Ok(value)
}

fn read_arr<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Vec<Value<'i>>, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.context("json array", |r| {
        let mut items = Vec::new();
        r.consume(b'[')?;
        skip_whitespace(r);
        if !r.peek_eq(b']') {
            loop {
                let val = read_value(r)?;
                skip_whitespace(r);
                items.push(val);
                if !r.at_end() && r.peek_eq(b',') {
                    r.skip(1)?;
                    continue;
                } else {
                    break;
                }
            }
        }
        skip_whitespace(r);
        r.consume(b']')?;
        Ok(items)
    })
}

fn read_map<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Vec<(&'i str, Value<'i>)>, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.context("json object", |r| {
        let mut items = Vec::new();
        r.consume(b'{')?;
        skip_whitespace(r);
        if !r.peek_eq(b'}') {
            loop {
                let key = r.context("json object key", read_str)?;
                skip_whitespace(r);
                r.consume(b':')?;
                skip_whitespace(r);
                let val = read_value(r)?;
                skip_whitespace(r);
                items.push((key, val));
                if !r.at_end() && r.peek_eq(b',') {
                    r.skip(1)?;
                    continue;
                } else {
                    break;
                }
            }
        }
        r.consume(b'}')?;
        Ok(items)
    })
}

fn read_str<'i, E>(r: &mut BytesReader<'i, E>) -> Result<&'i str, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.context("json string", |r| {
        r.consume(b'"')?;
        let mut last_was_escape = false;
        let s = r.take_while(|c| match c {
            b'\\' => {
                last_was_escape = true;
                true
            }
            b'"' => {
                let should_continue = last_was_escape;
                last_was_escape = false;
                should_continue
            }
            _ => {
                last_was_escape = false;
                true
            }
        });
        r.consume(b'"')?;
        s.to_dangerous_str()
    })
}

fn read_null<'i, E>(r: &mut BytesReader<'i, E>) -> Result<(), E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.context("json null", |r| r.consume(b"null"))
}

fn read_bool<'i, E>(r: &mut BytesReader<'i, E>) -> Result<bool, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.try_expect("json boolean", |r| match r.peek_u8()? {
        b't' => r.consume(b"true").map(|()| Some(true)),
        b'f' => r.consume(b"false").map(|()| Some(false)),
        _ => Ok(None),
    })
}

fn read_num<'i, E>(r: &mut BytesReader<'i, E>) -> Result<f64, E>
where
    E: Error<'i>,
{
    skip_whitespace(r);
    r.context("json number", |r| {
        r.try_take_consumed(|r| {
            r.try_verify("first byte is digit", |r| {
                r.read_u8().map(|c| c.is_ascii_digit())
            })?;
            r.skip_while(|c: u8| c.is_ascii_digit() || c == b'.');
            Ok(())
        })?
        .into_string::<E>()?
        .into_external("f64", |i| i.as_dangerous().parse())
    })
}

fn skip_whitespace<E>(r: &mut BytesReader<'_, E>) {
    r.skip_while(|c: u8| c.is_ascii_whitespace());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_size() {
        // If true, we box Expected!
        assert!(core::mem::size_of::<Value<'_>>() < 128);
    }
}
