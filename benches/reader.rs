use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dangerous::{input, Invalid, Reader};

fn bench_consume(c: &mut Criterion) {
    c.bench_function("consume_u8_ok", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 1]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.consume_u8(1))
                .unwrap();
        })
    });
    c.bench_function("consume_u8_err", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 1]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.consume_u8(2))
                .unwrap_err();
        })
    });
    c.bench_function("consume_bytes_ok", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.consume(&[1, 1]))
                .unwrap();
        })
    });
    c.bench_function("consume_bytes_err", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.consume(&[2, 2]))
                .unwrap_err();
        })
    });
}

fn bench_read_num(c: &mut Criterion) {
    c.bench_function("read_u16_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 2]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.read_u16_le())
                .unwrap();
        })
    });

    c.bench_function("read_u32_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 4]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.read_u32_le())
                .unwrap();
        })
    });

    c.bench_function("read_u64_le", |b| {
        b.iter(|| {
            input(black_box(&[1u8; 8]))
                .read_all(|r: &mut Reader<'_, Invalid>| r.read_u64_le())
                .unwrap();
        })
    });
}

criterion_group!(benches, bench_consume, bench_read_num);
criterion_main!(benches);
