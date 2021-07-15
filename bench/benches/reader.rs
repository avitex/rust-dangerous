use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dangerous::{input, BytesReader, Input, Invalid};

fn bench_consume(c: &mut Criterion) {
    c.bench_function("consume_u8_ok", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 1]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.consume(1))
                .unwrap();
        })
    });
    c.bench_function("consume_u8_err", |b| {
        b.iter(|| {
            let _ = input(black_box(&[1u8; 1]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.consume(2))
                .unwrap_err();
        })
    });
    c.bench_function("consume_bytes_ok", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.consume(&[1, 1]))
                .unwrap();
        })
    });
    c.bench_function("consume_bytes_err", |b| {
        b.iter(|| {
            let _ = input(black_box(&[1u8; 2]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.consume(&[2, 2]))
                .unwrap_err();
        })
    });
}

fn bench_read_num(c: &mut Criterion) {
    c.bench_function("read_u16_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.read_array().map(u16::from_le_bytes))
                .unwrap();
        })
    });

    c.bench_function("read_u32_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 4]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.read_array().map(u32::from_le_bytes))
                .unwrap();
        })
    });

    c.bench_function("read_u64_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 8]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| r.read_array().map(u64::from_le_bytes))
                .unwrap();
        })
    });
}

fn bench_peek_eq(c: &mut Criterion) {
    c.bench_function("peek_eq", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut BytesReader<'_, Invalid>| {
                    if r.peek_eq(&[1]) {
                        r.skip(2)
                    } else {
                        r.skip(0)
                    }
                })
                .unwrap();
        })
    });

    c.bench_function("peek_u8_eq", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(
                    |r: &mut BytesReader<'_, Invalid>| {
                        if r.peek_eq(1) {
                            r.skip(2)
                        } else {
                            r.skip(0)
                        }
                    },
                )
                .unwrap();
        })
    });
}

criterion_group!(benches, bench_peek_eq, bench_consume, bench_read_num);
criterion_main!(benches);
