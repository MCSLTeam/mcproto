use std::{hint::black_box, io::Cursor};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use mcproto_codec::varlong::{VarLongRead, VarLongWrite};

const ELEMENTS: usize = 10_000;
const VALUE_SEED: u64 = 0x51A7_EF00_89AB_CDEF;
const ERROR_SEED: u64 = 0xDEAD_BEEF_7654_3210;
const INVALID_VARLONG: [u8; 10] = [0xFF; 10];

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

fn values() -> Vec<i64> {
    let mut rng = SplitMix64(VALUE_SEED);
    let mut values: Vec<i64> = (0..ELEMENTS).map(|_| rng.next() as i64).collect();

    values[..5].copy_from_slice(&[i64::MIN, -1, 0, 1, i64::MAX]);
    values
}

fn encode(values: &[i64]) -> Vec<u8> {
    let mut output = Vec::with_capacity(values.len() * 10);
    for &value in values {
        output.write_varlong(black_box(value)).unwrap();
    }
    output
}

fn inputs_with_errors(values: &[i64]) -> Vec<Vec<u8>> {
    let mut rng = SplitMix64(ERROR_SEED);

    values
        .iter()
        .map(|&value| {
            if rng.next() % 100 < 5 {
                INVALID_VARLONG.to_vec()
            } else {
                let mut encoded = Vec::with_capacity(10);
                encoded.write_varlong(value).unwrap();
                encoded
            }
        })
        .collect()
}

fn benchmarks(c: &mut Criterion) {
    let values = values();
    let inputs_with_errors = inputs_with_errors(&values);
    let mut group = c.benchmark_group("varlong");
    group.throughput(Throughput::Elements(ELEMENTS as u64));

    group.bench_function("encode", |b| {
        b.iter(|| black_box(encode(black_box(&values))))
    });

    group.bench_function("decode_with_errors", |b| {
        b.iter(|| {
            for input in black_box(&inputs_with_errors) {
                let mut cursor = Cursor::new(input.as_slice());
                let _ = black_box(cursor.read_varlong());
            }
        })
    });

    group.bench_function("roundtrip", |b| {
        b.iter(|| {
            let encoded = encode(black_box(&values));
            let mut cursor = Cursor::new(encoded.as_slice());

            for _ in 0..ELEMENTS {
                black_box(cursor.read_varlong().unwrap());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
