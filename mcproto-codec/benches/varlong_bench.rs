use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rand::RngExt;
use mcproto_codec::*;
fn varlong_bench(data_vec: &Vec<i64>) {
    let mut buffer = Vec::with_capacity(10);
    for v in data_vec {
        buffer.write_varlong(*v).unwrap();
        buffer.as_slice().read_varlong().unwrap();
        buffer.clear();
    }
}
fn bench(c: &mut Criterion) {
    let vec: Vec<i64> = std::iter::repeat_with(|| rand::rng().random_range(i64::MIN..=i64::MAX))
        .take(1_000_000)
        .collect();

    let mut group = c.benchmark_group("varlong");
    group.throughput(Throughput::Elements(1_000_000));

    group.bench_function("write_read", |b| {
        b.iter(|| varlong_bench(black_box(&vec)))
    });

    group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);