## mcproto-codec
VarInt, encryption and compression coder & decoder for Minecraft protocols.

### Performance

Measured on CachyOS (Linux 6.12, x86_64-v4) with `target-cpu=native`:

| Type    | Throughput        | Latency  |
| ------- | ----------------- | -------- |
| VarInt  | **106 Melem/s**   | 9.4 ns   |
| VarLong | **50 Melem/s**    | 20 ns    |

Benchmarks powered by [Criterion](https://crates.io/crates/criterion).

### Examples
#### Run Benchmarks
```shell
cargo bench
```
#### Reading and writing VarInts
```rust

use mcproto_codec::*;
fn main() {
    let mut buf: Vec<u8> = Vec::new();
    // Write
    buf.write_varint(2147483647).unwrap();
    // Read
    let value: i32 = buf.as_slice().read_varint().unwrap();
    assert_eq!(value, 2147483647);
}
```

### Links
[Repository](https://github.com/MCSLTeam/mcproto)  
[Team's Github Page](https://github.com/MCSLTeam)
