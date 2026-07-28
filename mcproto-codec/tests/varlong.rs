//! Integration tests for VarLong encoding and decoding.
//!
//! Each case checks the bytes produced by the writer, decoding from known
//! bytes, and a round trip through both APIs. Expected encodings come from the
//! [Minecraft protocol VarLong definition].
//!
//! [Minecraft protocol VarLong definition]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong

use mcproto_codec::varlong::{VarLongRead, VarLongWrite};

// Generate the same three assertions for every value/encoding fixture.
macro_rules! varlong_case {
    ($name:ident, $value:expr, [$($byte:expr),+ $(,)?]) => {
        #[test]
        fn $name() {
            let expected = [$($byte),+];

            let mut encoded = Vec::new();
            encoded.write_varlong($value).unwrap();
            assert_eq!(encoded.as_slice(), expected);

            let mut input = expected.as_slice();
            assert_eq!(input.read_varlong().unwrap(), $value);

            let mut roundtrip = encoded.as_slice();
            assert_eq!(roundtrip.read_varlong().unwrap(), $value);
        }
    };
}

// Single-byte values and the first encoded-size boundary.
varlong_case!(zero, 0, [0x00]);
varlong_case!(one, 1, [0x01]);
varlong_case!(two, 2, [0x02]);
varlong_case!(one_byte_max, 127, [0x7f]);
varlong_case!(two_byte_min, 128, [0x80, 0x01]);

// Representative positive multi-byte values and upper boundaries.
varlong_case!(two_byte_value, 255, [0xff, 0x01]);
varlong_case!(i32_max, 2147483647, [0xff, 0xff, 0xff, 0xff, 0x07]);
varlong_case!(
    i64_max,
    9223372036854775807,
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]
);

// Negative values across both the i32 and i64 ranges.
varlong_case!(
    negative_one,
    -1,
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
);
varlong_case!(
    negative_i32_min,
    -2147483648,
    [0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]
);

// The lower bound of the represented Rust type.
varlong_case!(
    i64_min,
    -9223372036854775808,
    [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]
);
