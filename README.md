[![Build Status](https://github.com/avitex/rust-dangerous/workflows/build/badge.svg)](https://github.com/avitex/rust-dangerous/actions?query=workflow:build)
[![Coverage Status](https://codecov.io/gh/avitex/rust-dangerous/branch/master/graph/badge.svg?token=X2LXHI8VYL)](https://codecov.io/gh/avitex/rust-dangerous)
[![Crate](https://img.shields.io/crates/v/dangerous.svg)](https://crates.io/crates/dangerous)
[![Docs](https://docs.rs/dangerous/badge.svg)](https://docs.rs/dangerous)

# rust-dangerous

**Rust library for safely and explicitly handling untrusted aka `dangerous` data**  
Documentation hosted on [docs.rs](https://docs.rs/dangerous).

```toml
dangerous = "0.9"
```

## Goals

- Fast parsing.
- Fast to compile.
- Zero panics \[1].
- Zero-cost abstractions.
- Minimal dependencies \[2].
- Retry/stream protocol support.
- `no-std` / suitable for embedded.
- Zero heap-allocations on success paths \[3].
- Primitive type support.
- Optional verbose errors.
- Optional SIMD optimisations where possible.

**\[1]** Panics due to OOM are out-of-scope. Disable heap-allocations if this is
a concern.  
**\[2]** Zero dependencies when both `unicode` and `simd` features are disabled.  
**\[3]** Zero heap-allocations when the `full-backtrace` feature is disabled.

This library's intentions are to provide a simple interface for explicitly
parsing untrusted data safely. `dangerous` really shines with parsing binary or
simple text data formats and protocols. It is not a deserialisation library like
what `serde` provides, but you could write a parser with `dangerous` that could
be used within a deserialiser.

Panics and unhandled/unacknowledged data are two footguns this library seeks to
prevent. An optional, but solid, debugging interface with sane input formatting
and helpful errors is included to weed out problems before, or after they arise
in production.

## Usage

```rust
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

let input = dangerous::input(/* data */);
let result: Result<_, Invalid> = input.read_all(decode_message);
```

## Errors

Custom errors for protocols often do not provide much context around why and
where a specific problem occurs within input. Passing down errors as simple as
`core::str::Utf8Error` may be useful enough to debug while in development,
however when just written into logs without the input/context, often amount to
noise. At this stage you are almost better off with a simple input error.

This problem is amplified with any trivial recursive-descent parser as the
context around a sub-slice is lost, rendering any error offsets useless when
passed back up to the root. `dangerous` fixes this by capturing the context
around and above the error.

Ever tried working backwards from something like this?

```
[ERRO]: ahhh!: Utf8Error { valid_up_to: 2, error_len: Some(1) }
```

Wouldn't it be better if this was the alternative?

```
[ERRO]: ahhh!: error reading message: error attempting to convert input into string: expected utf-8 code point
> [01 05 68 65 ff 6c 6f]
               ^^       
additional:
  error offset: 4, input length: 7
backtrace:
  1. `read all input`
  2. `<context>` (expected message)
  3. `<context>` (expected body)
  4. `convert input into string` (expected utf-8 code point)
```

## Inspiration

This project was originally inspired by [untrusted](https://github.com/briansmith/untrusted).
