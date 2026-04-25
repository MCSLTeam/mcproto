use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rand::RngExt;
use mcproto_codec::*;
fn varint_bench(data_vec: &Vec<i32>) {
    let mut buffer = Vec::with_capacity(5);
    for v in data_vec {
        buffer.write_varint(*v).unwrap();
        buffer.as_slice().read_varint().unwrap();
        buffer.clear();
    }
}
fn bench(c: &mut Criterion) {
    let vec: Vec<i32> = std::iter::repeat_with(|| rand::rng().random_range(i32::MIN..=i32::MAX))
        .take(1_000_000)
        .collect();

    let mut group = c.benchmark_group("varint");
    group.throughput(Throughput::Elements(1_000_000));

    group.bench_function("write_read", |b| {
        b.iter(|| varint_bench(black_box(&vec)))
    });

    group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);