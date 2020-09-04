[![Build Status](https://travis-ci.com/avitex/rust-dangerous.svg?branch=master)](https://travis-ci.com/avitex/rust-dangerous)
[![Crate](https://img.shields.io/crates/v/dangerous.svg)](https://crates.io/crates/dangerous)
[![Docs](https://docs.rs/dangerous/badge.svg)](https://docs.rs/dangerous)

# rust-dangerous (unpublished)

**Rust library for safely and explicitly handling user-generated aka `dangerous` data**  
Documentation hosted on [docs.rs](https://docs.rs/dangerous).

```toml
dangerous = "0.1"
```

## TODO

- [ ] Documentation
- [ ] Unit test coverage
- [ ] Review `must_use` and `inline` usage
- [ ] Error display
- [ ] Input display
- [ ] Stabilize errors
- [ ] Stabilize `Reader` interface
- [ ] Terminal support

## Goals

- Fast.
- Zero panics.
- Zero heap-allocations.
- Zero-cost abstractions.
- Minimal dependencies \[1].
- Primitive type support.
- Optional verbose errors.

\[1] Zero-dependencies if the `unicode` feature is disabled.

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
[ERRO]: ahhh!: invalid utf-8 code point
> [.. "aaaa" ff "aaaa" ..]
             ^^
```

## Safety

This library has one instance of `unsafe` required for wrapping a
byte slice into the `Input` DST.

**No other instances of `unsafe` are permitted.**

## Inspiration

This project was inspired by [untrusted](https://github.com/briansmith/untrusted).
