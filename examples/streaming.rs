//! This example demonstrates a protocol being read from a stream.
//!
//! The simple protocol encodes messages with a single byte for versioning
//! followed by a single byte that denotes the the UTF-8 body length we need to
//! read. Our protocol expects a version of `1`.

// FIXME: This example requires `RUSTFLAGS=-Zpolonius` to run do to the mut ref
// reuse within the a loop.
//
// ```
// RUSTFLAGS=-Zpolonius cargo run --example streaming --features std
// ```

use std::error::Error;
use std::io;

use dangerous::Expected;

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
    let mut buf = [0u8; 256];

    // Read a valid message
    let message = read_and_decode_message(&mut Stream::new(VALID_MESSAGE), &mut buf[..]).unwrap();

    println!("{}", message.body);

    // Read a invalid message
    let err = read_and_decode_message(&mut Stream::new(INVALID_MESSAGE), &mut buf[..]).unwrap_err();

    eprintln!("error reading message: {}", err);
}

fn read_and_decode_message<'i, R>(
    read: &mut R,
    buf: &'i mut [u8],
) -> Result<Message<'i>, Box<dyn Error + 'i>>
where
    R: io::Read,
{
    let mut written_cur = 0;
    let mut expects_cur = 0;
    loop {
        // Read bytes into buffer
        written_cur += read.read(&mut buf[written_cur..])?;
        // Only decode the buffer if we have enough bytes to try again
        if expects_cur > written_cur {
            println!(
                "not enough to decode, waiting for {} bytes",
                expects_cur - written_cur
            );
            continue;
        }
        let input = dangerous::input(&buf[..written_cur]);
        match decode_message::<Expected>(input) {
            Err(err) => match err.can_continue_after() {
                Some(len) => expects_cur += len,
                None => return Err(err.into()),
            },
            Ok(message) => {
                return Ok(message);
            }
        }
    }
}

fn decode_message<'i, E>(input: &'i dangerous::Input) -> Result<Message<'i>, E>
where
    E: From<dangerous::ExpectedLength<'i>>,
    E: From<dangerous::ExpectedValid<'i>>,
    E: From<dangerous::ExpectedValue<'i>>,
{
    let mut r = input.reader::<E>();
    // Expect version 1
    r.consume(&[0x01])?;
    // Read the body length
    let body_len = r.read_u8()?;
    // Take the body input
    let body_input = r.take(body_len as usize)?;
    // Decode the body input as a UTF-8 str
    let body = body_input.to_dangerous_str::<E>()?;
    // We did it!
    Ok(Message { body })
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
