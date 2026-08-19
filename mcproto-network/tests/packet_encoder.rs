//! Integration tests for named packet encoding.

use mcproto_network::{
    CompressionCodec, CompressionError, CompressionMode, EncryptionError, EncryptionMode,
    PacketCodec, PacketEncoder, PacketLimits, StreamEncryptor,
};

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
fn compression_precedes_encryption_and_includes_length() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::enabled(0, MarkerCompression),
        EncryptionMode::enabled(Xor(0xff)),
        PacketLimits::default(),
    );
    // Plain frame is [3, 1, 0xc0, 1]; every byte is then XORed.
    assert_eq!(encoder.encode(&Empty).unwrap(), [0xfc, 0xfe, 0x3f, 0xfe]);
}
