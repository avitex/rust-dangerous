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

- Fast.
- Zero panics.
- Zero-cost abstractions.
- Zero heap-allocations on success \[1].
- `no-std` / suitable for embedded.
- Retry/stream protocol support.
- Minimal dependencies \[2].
- Primitive type support.
- Optional verbose errors.

**\[1]** Allocations for error cases (`alloc` feature is recommended for better
perf).  
**\[2]** Zero-dependencies if the `unicode` feature is disabled.

This library's intentions are to provide a simple interface for explicitly
parsing untrusted data safely. It tries to achieve this by providing useful
primitives for parsing data and an optional, but solid, debugging interface with
sane input formatting and errors to weed out problems before, or after they
arise in production.

Passing down errors as simple as `core::str::Utf8Error` may be useful enough to
debug while in development, however when just written into logs without the
input/context, often amount to noise. At this stage you are almost better off
with a simple input error.

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
used in display section.

## Inspiration

This project was originally inspired by [untrusted](https://github.com/briansmith/untrusted).
