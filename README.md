[![Build Status](https://travis-ci.com/avitex/rust-dangerous.svg?branch=master)](https://travis-ci.com/avitex/rust-dangerous)
[![Coverage Status](https://coveralls.io/repos/github/avitex/rust-dangerous/badge.svg)](https://coveralls.io/github/avitex/rust-dangerous)
[![Crate](https://img.shields.io/crates/v/dangerous.svg)](https://crates.io/crates/dangerous)
[![Docs](https://docs.rs/dangerous/badge.svg)](https://docs.rs/dangerous)

# rust-dangerous

**Rust library for safely and explicitly handling untrusted aka `dangerous` data**  
Documentation hosted on [docs.rs](https://docs.rs/dangerous).

```toml
dangerous = "0.1"
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

**\[1]** Panics due to OOM are out-of-scope. Disable heap-allocations if this is
a concern.  
**\[2]** Zero dependencies when both `unicode` and `bytecount`
features are disabled.  
**\[3]** Zero heap-allocations when both `box-expected` and `full-context`
features are disabled.

This library's intentions are to provide a simple interface for explicitly
parsing untrusted data safely. `dangerous` really shines with parsing binary or
simple text data formats and protocols. It is not a deserialisation library like
what `serde` provides, but you could write a parser with `dangerous` that could
be used within a deserialiser.

Panics and unhandled/unacknowledged data are two footguns this library seeks to
prevent. An optional, but solid, debugging interface with sane input formatting
and helpful errors is included to weed out problems before, or after they arise
in production.

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
[ERRO]: ahhh!: error attempting to convert input to str: expected utf-8 code point
> ['h' 'e' ff 'l' 'o']
           ^^
additional:
  error offset: 2, input length: 5
backtrace:
  1. `read all`
  2. `read` (expected message)
  3. `read` (expected body)
  4. `convert input to str` (expected utf-8 code point)
```

## Safety

This library has a instance of `unsafe` required for wrapping a byte slice into
the `Input` DST and multiple instances required for `str::from_utf8_unchecked`
used in the display section module.

## Inspiration

This project was originally inspired by [untrusted](https://github.com/briansmith/untrusted).
