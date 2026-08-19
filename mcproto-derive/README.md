# mcproto-derive

Derive macros for `mcproto-types` and `mcproto-network`.

## `PacketCodec`

`PacketCodec` implements static `Packet` metadata plus direction-specific body
coding for a specific protocol version. Serverbound packets implement
`EncodePacket`; clientbound packets implement `DecodePacket`:

```rust
use mcproto_network::PacketCodec;
use mcproto_types::VarInt;

#[derive(PacketCodec)]
#[packet(
    name = "example",
    id = 0x01,
    state = Play,
    direction = Clientbound,
)]
struct ExamplePacket {
    value: VarInt,
}
```

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

## `TypeStructCodec`

`TypeStructCodec` derives a sequential [`TypeCodec`] implementation for named,
tuple, unit, and generic structs. Fields are encoded in declaration order and
must implement `TypeCodec`.

```rust
use mcproto_types::{TypeStructCodec, VarInt};

#[derive(TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
struct Pair {
    first: VarInt,
    second: VarInt,
}
```
