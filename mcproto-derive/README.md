# mcproto-derive

Derive macros for `mcproto-types`.

## `ProtocolEnum`

`ProtocolEnum` derives a numeric Minecraft protocol enum codec for a fieldless
Rust enum:

```rust
use mcproto_types::{ProtocolEnum, TypeCodec, basic::VarInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
enum GameMode {
    Survival = 0,
    Creative = 1,
}

let mut encoded = Vec::new();
GameMode::Creative.encode(&mut encoded)?;
assert_eq!(encoded, [0x01]);
# Ok::<(), mcproto_codec::error::CodecError>(())
```
