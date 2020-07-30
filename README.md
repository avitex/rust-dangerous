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

- [ ] Enable `missing_docs`
- [ ] Review `must_use` usage
- [ ] Finish impl error system
- [ ] Stabilize `Reader` interface
- [ ] More documentation and tests

## Goals

- Fast.
- Zero panics.
- Zero dependencies.
- Zero heap-allocations.
- Zero-cost abstractions.
- Primitive type support.
- Optional verbose errors.

## Safety

This library has one instance of `unsafe` required for wrapping a
byte slice into the `Input` DST.

**No other instances of `unsafe` are permitted.**

## Inspiration

This project was inspired by [untrusted](https://github.com/briansmith/untrusted).
