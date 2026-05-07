## mcproto-types
All types for Minecraft.

### Basic Types
Basic types are in `basic` mod.

| Type            | Notes                                                               | Protocol Name  |
|-----------------|---------------------------------------------------------------------|----------------|
| `bool`          | True is encoded as `0x01`, false as `0x00`.                         | Boolean        |
| `Byte`          | An integer between `-128` and `127`                                 | Byte           |
| `UnsignedByte`  | An integer between `0` and `255`                                    | Unsigned Byte  |
| `Short`         | An integer between `-32768` and `32767`                             | Short          |
| `UnsignedShort` | An integer between `0` and `65535`                                  | Unsigned Short |
| `Int`           | An integer between `-2147483648` and `2147483647`                   | Int            |
| `Long`          | An integer between `-9223372036854775808` and `9223372036854775807` | Long           |
| `Float`         | A single-precision 32-bit IEEE 754 floating point number            | Float          |
| `Double`        | A double-precision 64-bit IEEE 754 floating point number            | Double         |
| `String`        | UTF-8 string prefixed with its length as a `VarInt`                 | String         |
| `Identifier`    | String wrapper for namespaced identifiers, e.g. `minecraft:stone`   | Identifier     |
| `VarInt`        | Variable-length integer (`-2147483648` to `2147483647`, 1-5 bytes)  | VarInt         |
| `VarLong`       | Variable-length long (`-2^63` to `2^63-1`, 1-10 bytes)              | VarLong        |

### Compound Types
Compound types are in `compound` mod.

| Type                | Notes                                                                                                                                           | Protocol Name  |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|----------------|
| `Angle`             | Rotation angle in steps of `1/256` of a full turn (`0-255 = 0-360°`)                                                                            | Angle          |
| `TextComponent`     | UTF-8 text encoded as a `VarInt`-prefixed string                                                                                                | Text           |
| `JsonTextComponent` | JSON text component encoded as a `VarInt`-prefixed UTF-8 string; decode max `262144` chars, encode max `32767` chars                            | Text Component |
| `Position`          | Packed block position: `x/z` use 26 signed bits, `y` uses 12 signed bits                                                                        | Position       |
| `UUID`              | UUID encoded as 16 bytes: most significant 64 bits followed by least significant 64 bits                                                        | UUID           |
| `LpVec3`            | Low-precision 3D vector for velocity fields; quantized and packed (usually 6 bytes, may include VarInt scale extension)                         | LpVec3         |
| `BitSet`            | Length-prefixed bit set: `VarInt` long-count followed by that many signed 64-bit big-endian `Long` values                                       | BitSet         |
| `FixedBitSet<N>`    | Fixed-length bit set encoded as exactly `N` bytes (`ceil(bits/8)`), using per-byte LSB-first bit ordering                                       | Fixed BitSet   |
| `Nbt`               | The Named Binary Tag (NBT) file format is an extremely simple and efficient structured binary format used by Minecraft for a variety of things. | NBT            |


### Contextual Types
Contextual types are in `contextual` mod and use `ContextualCodec` with `Ctx`.

| Type                  | Notes                                                                                                                | Protocol Name       |
|-----------------------|----------------------------------------------------------------------------------------------------------------------|---------------------|
| `Optional<T>`         | Optional value without prefix; encoded as either nothing or `T`. Presence is controlled by `Ctx.present`.            | Optional X          |
| `PrefixedOptional<T>` | Optional value with boolean prefix; `false` encodes only the prefix, `true` encodes prefix + `T`.                    | Prefixed Optional X |
| `Array<T>`            | Non-prefixed array of `T`; length is provided by context via `Ctx.len`.                                              | Array of X          |
| `PrefixedArray<T>`    | Length-prefixed array of `T`; encoded as `VarInt` length followed by elements.                                       | Prefixed Array of X |
| `ByteArray`           | Raw sequence of bytes with length provided by context via `Ctx.len` (no self length prefix).                         | Byte Array          |
| `IdOr<T>`             | Either registry reference or inline `T`: `id=0` means inline value follows, otherwise registry id is `id-1`.         | ID or X             |
| `IdSet`               | Set of registry IDs represented either by tag (`type=0` + `Identifier`) or inline IDs (`type=len+1` + VarInt array). | ID Set              |
