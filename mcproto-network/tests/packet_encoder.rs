//! Integration tests for named packet encoding.

use mcproto_network::{
    CompressionCodec, CompressionError, CompressionMode, EncryptionError, EncryptionMode,
    PacketCodec, PacketEncoder, PacketLimits, StreamEncryptor,
};
use mcproto_types::FixedByteArray;

struct MarkerCompression;

impl CompressionCodec for MarkerCompression {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut output = vec![0xC0];
        output.extend_from_slice(input);
        Ok(output)
    }

    fn decompress(
        &mut self,
        input: &[u8],
        _expected_len: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        Ok(input[1..].to_vec())
    }
}

struct Xor(u8);

impl StreamEncryptor for Xor {
    fn encrypt(&mut self, data: &mut [u8]) -> Result<(), EncryptionError> {
        for byte in data {
            *byte ^= self.0;
        }
        Ok(())
    }
}

#[derive(PacketCodec)]
#[packet(
    name = "empty_test",
    id = 0x01,
    state = Status,
    direction = Serverbound
)]
struct Empty;

#[derive(PacketCodec)]
#[packet(
    name = "large_empty_test",
    id = 0x01,
    state = Status,
    direction = Serverbound
)]
struct LargeEmpty {
    data: FixedByteArray<127>,
}

#[test]
fn uncompressed_unencrypted_frame() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::disabled(),
        EncryptionMode::disabled(),
        PacketLimits::default(),
    );
    assert_eq!(encoder.encode(&Empty).unwrap(), [1, 1]);
}

#[test]
fn uncompressed_frame_rewrites_a_two_byte_length_prefix() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::disabled(),
        EncryptionMode::disabled(),
        PacketLimits::default(),
    );
    let packet = LargeEmpty {
        data: FixedByteArray([0x5a; 127]),
    };

    let frame = encoder.encode(&packet).unwrap();

    assert_eq!(&frame[..3], [0x80, 0x01, 0x01]);
    assert_eq!(&frame[3..], [0x5a; 127]);
}

#[test]
fn encode_into_reuses_the_output_allocation() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::disabled(),
        EncryptionMode::disabled(),
        PacketLimits::default(),
    );
    let mut output = Vec::with_capacity(1024);

    encoder.encode_into(&Empty, &mut output).unwrap();
    let allocation = output.as_ptr();
    assert_eq!(output, [1, 1]);

    encoder.encode_into(&Empty, &mut output).unwrap();
    assert_eq!(output, [1, 1]);
    assert_eq!(output.as_ptr(), allocation);
}

#[test]
fn compression_precedes_encryption_and_includes_length() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::enabled(0, MarkerCompression),
        EncryptionMode::enabled(Xor(0xff)),
        PacketLimits::default(),
    );
    // Plain frame is [3, 1, 0xc0, 1]; every byte is then XORed.
    assert_eq!(encoder.encode(&Empty).unwrap(), [0xfc, 0xfe, 0x3f, 0xfe]);
}
