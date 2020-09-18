use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dangerous::{Error, Expected, Invalid};

fn expected<'i, E>(bytes: &'i [u8]) -> Result<(), E>
where
    E: Error<'i>,
{
    dangerous::input(bytes).read_all(|r| {
        r.context("foo", |r| {
            r.context("bar", |r| {
                r.context("hello", |r| r.context("world", |r| r.consume(b"2")))
            })
        })
    })
}

fn bench_contexts(c: &mut Criterion) {
    // Invalid
    c.bench_function("invalid failure", |b| {
        b.iter(|| expected::<Invalid>(black_box(b"1")))
    });
    c.bench_function("invalid success", |b| {
        b.iter(|| expected::<Invalid>(black_box(b"2")))
    });
    // Expected
    c.bench_function("expected failure", |b| {
        b.iter(|| expected::<Expected>(black_box(b"1")))
    });
    c.bench_function("expected success", |b| {
        b.iter(|| expected::<Expected>(black_box(b"2")))
    });
}

criterion_group!(benches, bench_contexts);
criterion_main!(benches);
