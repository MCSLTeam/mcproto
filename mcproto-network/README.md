# mcproto-network

Minecraft protocol framing and networking primitives.

Packet types declare the numeric ID for the protocol version they implement:

```rust
use mcproto_network::{
    CompressionMode, EncryptionMode, PacketCodec, PacketEncoder, PacketLimits,
};

#[derive(PacketCodec)]
#[packet(
    name = "status_request",
    id = 0x00,
    state = Status,
    direction = Serverbound,
)]
struct StatusRequest;

let mut encoder = PacketEncoder::new(
    CompressionMode::disabled(),
    EncryptionMode::disabled(),
    PacketLimits::default(),
);

let frame = encoder.encode(&StatusRequest)?;
assert_eq!(frame, [0x01, 0x00]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Packet IDs vary between protocol versions, so packet modules should be scoped
to one version. Enabling compression or encryption changes only the
corresponding encoder mode. Serverbound packet derives can be encoded, while
clientbound packet derives can only decode their packet body.

## License

Licensed under the [MIT License](LICENSE).
