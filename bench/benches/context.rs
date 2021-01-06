use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dangerous::{Error, Expected, Invalid};

fn expected<'i, E>(bytes: &'i [u8]) -> Result<(), E>
where
    E: Error<'i>,
{
    dangerous::input(bytes).read_all(|r| {
        r.context("foo", |r| {
            r.context("bar", |r| {
                r.context("hello", |r| r.context("world", |r| r.consume(b"o")))
            })
        })
    })
}

fn bench_invalid(c: &mut Criterion) {
    c.bench_function("invalid_ok", |b| {
        b.iter(|| expected::<Invalid>(black_box(b"o")))
    });
    c.bench_function("invalid_err", |b| {
        b.iter(|| expected::<Invalid>(black_box(b"e")))
    });
}

fn bench_expected(c: &mut Criterion) {
    c.bench_function("expected_ok", |b| {
        b.iter(|| expected::<Expected>(black_box(b"o")))
    });
    c.bench_function("expected_err", |b| {
        b.iter(|| expected::<Expected>(black_box(b"e")))
    });
    c.bench_function("expected_ok_boxed", |b| {
        b.iter(|| expected::<Box<Expected>>(black_box(b"o")))
    });
    c.bench_function("expected_err_boxed", |b| {
        b.iter(|| expected::<Box<Expected>>(black_box(b"e")))
    });
}

criterion_group!(benches, bench_invalid, bench_expected);
criterion_main!(benches);
