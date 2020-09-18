[![Build Status](https://travis-ci.com/avitex/rust-dangerous.svg?branch=master)](https://travis-ci.com/avitex/rust-dangerous)
[![Coverage Status](https://coveralls.io/repos/github/avitex/rust-dangerous/badge.svg)](https://coveralls.io/github/avitex/rust-dangerous)
[![Crate](https://img.shields.io/crates/v/dangerous.svg)](https://crates.io/crates/dangerous)
[![Docs](https://docs.rs/dangerous/badge.svg)](https://docs.rs/dangerous)

# rust-dangerous (unpublished)

**Rust library for safely and explicitly handling user-generated aka `dangerous` data**  
Documentation hosted on [docs.rs](https://docs.rs/dangerous).

```toml
dangerous = "0.1"
```

## TODO

- [ ] Terminal support
- [ ] Unit test coverage
- [ ] Error and Input display
- [ ] Review `must_use` usage
- [ ] Review `Reader` interface

## Goals

- Fast.
- Zero panics.
- Zero-cost abstractions.
- Zero heap-allocations on success \[1].
- Minimal dependencies \[2].
- Primitive type support.
- Optional verbose errors.

**\[1]** Allocations for error cases (`alloc` feature is recommended for better perf).  
**\[2]** Zero-dependencies if the `unicode` feature is disabled.

This library intentions are to provide a simple interface for explicitly handling user-generated data safely.
It tries to achieve this by providing useful primitives for parsing data and an optional, but solid, debugging
interface with sane input formatting and errors to weed out problems before, or after they arise in production.

Passing down errors as simple as `core::str::Utf8Error` may be useful enough to debug while in development,
however when just written into logs without the input/context, often amount to noise. At this stage 
you are almost better off with a simple input error.

Ever tried working backwards from something like this?

```
[ERRO]: ahhh!: Utf8Error { valid_up_to: 42, error_len: Some(1) }
```

Wouldn't it be better if this was the alternative?

```
[ERRO]: ahhh!: expected utf-8 code point
> [.. "aaaa" ff "aaaa" ..]
             ^^
context bracktrace:
  1. `read all`
  2. `read` (expected message)
  3. `read` (expected body)
  4. `input to str` (expected valid utf-8 code point)
```

## Safety

This library has one instance of `unsafe` required for wrapping a
byte slice into the `Input` DST.

**No other instances of `unsafe` are permitted.**

## Inspiration

This project was inspired by [untrusted](https://github.com/briansmith/untrusted).
