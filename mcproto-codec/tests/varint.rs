//! Integration tests for VarInt encoding and decoding.
//!
//! Each case checks the bytes produced by the writer, decoding from known
//! bytes, and a round trip through both APIs. Expected encodings come from the
//! [Minecraft protocol VarInt definition].
//!
//! [Minecraft protocol VarInt definition]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong

use mcproto_codec::varint::{VarIntRead, VarIntWrite};

// Generate the same three assertions for every value/encoding fixture.
macro_rules! varint_case {
    ($name:ident, $value:expr, [$($byte:expr),+ $(,)?]) => {
        #[test]
        fn $name() {
            let expected = [$($byte),+];

            let mut encoded = Vec::new();
            encoded.write_varint($value).unwrap();
            assert_eq!(encoded.as_slice(), expected);

            let mut input = expected.as_slice();
            assert_eq!(input.read_varint().unwrap(), $value);

            let mut roundtrip = encoded.as_slice();
            assert_eq!(roundtrip.read_varint().unwrap(), $value);
        }
    };
}

// Single-byte values and the first encoded-size boundary.
varint_case!(zero, 0, [0x00]);
varint_case!(one, 1, [0x01]);
varint_case!(two, 2, [0x02]);
varint_case!(one_byte_max, 127, [0x7f]);
varint_case!(two_byte_min, 128, [0x80, 0x01]);

// Representative positive multi-byte values and upper boundaries.
varint_case!(two_byte_value, 255, [0xff, 0x01]);
varint_case!(minecraft_port, 25565, [0xdd, 0xc7, 0x01]);
varint_case!(three_byte_max, 2097151, [0xff, 0xff, 0x7f]);
varint_case!(i32_max, 2147483647, [0xff, 0xff, 0xff, 0xff, 0x07]);

// Negative values, including the lower bound of the represented Rust type.
varint_case!(negative_one, -1, [0xff, 0xff, 0xff, 0xff, 0x0f]);
varint_case!(i32_min, -2147483648, [0x80, 0x80, 0x80, 0x80, 0x08]);
