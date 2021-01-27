//! This example demonstrates a protocol being read from a stream.
//!
//! The simple protocol encodes messages with a single byte for versioning
//! followed by a single byte that denotes the the UTF-8 body length we need to
//! read. Our protocol expects a version of `1`.

use std::error::Error as StdError;
use std::io;

use dangerous::{BytesReader, Error, Expected, Input, Invalid, ToRetryRequirement};

const VALID_MESSAGE: &[u8] = &[
    0x01, // version: 1
    0x05, // body length: 5
    b'h', b'e', b'l', b'l', b'o', // the body value
];

const INVALID_MESSAGE: &[u8] = &[
    0x01, // version: 1
    0x05, // body length: 5
    b'h', b'e', 0xff, b'l', b'o', // the body value with invalid UTF-8
];

#[derive(Debug)]
struct Message<'a> {
    body: &'a str,
}

fn main() {
    let mut decoder = Decoder::new();

    // Read a valid message
    let message = decoder
        .read_and_decode_message(&mut Stream::new(VALID_MESSAGE))
        .unwrap();

    println!("{}", message.body);

    // Read a invalid message
    let err = decoder
        .read_and_decode_message(&mut Stream::new(INVALID_MESSAGE))
        .unwrap_err();

    eprintln!("error reading message: {}", err);
}

pub struct Decoder {
    buf: [u8; 256],
}

impl Decoder {
    fn new() -> Self {
        Self { buf: [0u8; 256] }
    }

    fn read_and_decode_message<'i, R>(
        &'i mut self,
        mut read: R,
    ) -> Result<Message<'i>, Box<dyn StdError + 'i>>
    where
        R: io::Read,
    {
        let mut written_cur = 0;
        let mut expects_cur = 0;
        loop {
            // Read bytes into buffer
            written_cur += read.read(&mut self.buf[written_cur..])?;
            // Only decode the buffer if we have enough bytes to try again
            if expects_cur > written_cur {
                println!(
                    "not enough to decode, waiting for {} bytes",
                    expects_cur - written_cur
                );
                continue;
            }
            // Try and decode the input, working out if we need more, or the
            // input is invalid.
            //
            // FIXME: This would realistically return the decoded message or any
            // error, but we can't mut borrow and return immutable within a loop
            // yet. See: https://github.com/rust-lang/rust/issues/51132
            let input = dangerous::input(&self.buf[..written_cur]);
            match input.read_all(decode_message) {
                Err(err) => match Invalid::to_retry_requirement(&err) {
                    Some(req) => {
                        expects_cur += req.continue_after();
                        continue;
                    }
                    None => break,
                },
                Ok(_) => break,
            }
        }
        // Decode the input returning the message or any error, see above why
        // this is required.
        dangerous::input(&self.buf[..written_cur])
            .read_all(decode_message)
            .map_err(Box::<Expected>::into)
    }
}

fn decode_message<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Message<'i>, E>
where
    E: Error<'i>,
{
    r.context("message", |r| {
        // Expect version 1
        r.context("version", |r| r.consume(0x01))?;
        // Read the body length
        let body_len = r.context("body len", |r| r.read_u8())?;
        // Take the body input
        let body = r.context("body", |r| {
            let body_input = r.take(body_len as usize)?;
            // Decode the body input as a UTF-8 str
            body_input.to_dangerous_str()
        })?;
        // We did it!
        Ok(Message { body })
    })
}

// Dummy reader that reads one byte at a time
struct Stream {
    cur: usize,
    src: &'static [u8],
}

impl Stream {
    fn new(src: &'static [u8]) -> Self {
        Self { src, cur: 0 }
    }
}

impl io::Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // If we have read all of the message, the connection is dead
        if self.cur == self.src.len() {
            return Err(io::Error::from(io::ErrorKind::NotConnected));
        }
        // Copy the byte across to the buffer at the cursor
        buf[0] = self.src[self.cur];
        // Increase the cursor for next invoke
        self.cur += 1;
        // Return the number of bytes we read
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_size() {
        // If true, we box Expected!
        assert!(core::mem::size_of::<Message<'_>>() < 128);
    }
}
