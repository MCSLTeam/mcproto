use mcproto_codec::*;
use rand::RngExt;
use std::time::Instant;

#[test]
fn test_varlong_roundtrip() {
    let iterations = 100_000_0;
    let mut rng = rand::rng();

    // 预先生成随机数
    let values: Vec<i64> = (0..iterations)
        .map(|_| rng.random())
        .collect();

    let start = Instant::now();

    let mut buf = Vec::with_capacity(10);  // VarLong 最大 10 字节

    for &value in &values {
        buf.clear();
        buf.write_varlong(value).unwrap();

        let mut reader = &buf[..];
        let decoded = reader.read_varlong().unwrap();
        assert_eq!(value, decoded);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() / iterations;
    let throughput = iterations as f64 / duration.as_secs_f64();
    println!("\n=== VarLong Roundtrip Full ===");
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", duration);
    println!("Average: {} ns/op", avg_ns);
    println!("Throughput: {:.0} ops/s", throughput);
}