## mcproto-types
All types for Minecraft.

### Basic Types
Basic types are in `basic` mod.

| Type            | Notes                                                           | Protocol Name  |
|-----------------|-----------------------------------------------------------------|----------------|
| `bool`          | True is encoded as 0x01, false as 0x00.                         | Boolean        |
| `Byte`          | An integer between -128 and 127                                 | Byte           |
| `UnsignedByte`  | An integer between 0 and 255                                    | Unsigned Byte  |
| `Short`         | An integer between -32768 and 32767                             | Short          |
| `UnsignedShort` | An integer between 0 and 65535                                  | Unsigned Short |
| `Int`           | An integer between -2147483648 and 2147483647                   | Int            |
| `Long`          | An integer between -9223372036854775808 and 9223372036854775807 | Long           |
| `Float`         | A single-precision 32-bit IEEE 754 floating point number        | Float          |
| `Double`        | A single-precision 64-bit IEEE 754 floating point number        | Double         |