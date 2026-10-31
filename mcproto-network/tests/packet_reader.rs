//! Integration tests for inbound compression and stream decryption.

use mcproto_network::{
    CompressionCodec, CompressionError, CompressionMode, DecryptionMode, EncryptionError,
    EncryptionMode, FrameError, NetworkError, PacketCodec, PacketEncoder, PacketReadLimits,
    PacketReader, StreamDecryptor, StreamEncryptor,
};
use mcproto_types::VarInt;

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
        expected_len: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        assert_eq!(input.first(), Some(&0xC0));
        let output = input[1..].to_vec();
        if output.len() != expected_len {
            return Err(CompressionError::LengthMismatch {
                declared: expected_len,
                actual: output.len(),
            });
        }
        Ok(output)
    }
}

#[derive(Clone, Copy)]
struct Xor(u8);

impl StreamEncryptor for Xor {
    fn encrypt(&mut self, data: &mut [u8]) -> Result<(), EncryptionError> {
        for byte in data {
            *byte ^= self.0;
        }
        Ok(())
    }
}

impl StreamDecryptor for Xor {
    fn decrypt(&mut self, data: &mut [u8]) -> Result<(), EncryptionError> {
        for byte in data {
            *byte ^= self.0;
        }
        Ok(())
    }
}

#[derive(PacketCodec)]
#[packet(
    name = "reader_test",
    id = 0x2a,
    state = Play,
    direction = Serverbound
)]
struct ReaderTest {
    value: VarInt,
}

fn packet() -> ReaderTest {
    ReaderTest { value: VarInt(300) }
}

#[test]
fn reads_an_uncompressed_plaintext_frame() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::disabled(),
        EncryptionMode::disabled(),
        mcproto_network::PacketLimits::default(),
    );
    let frame = encoder.encode(&packet()).unwrap();
    let mut reader = PacketReader::new(
        frame.as_slice(),
        CompressionMode::disabled(),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    );

    let payload = reader.read_packet().unwrap();

    assert_eq!(payload.packet_id.get(), 0x2a);
    assert_eq!(payload.body(), [0xac, 0x02]);
    assert_eq!(payload.into_body(), [0xac, 0x02]);
}

#[test]
fn reads_a_compressed_and_encrypted_frame() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::enabled(0, MarkerCompression),
        EncryptionMode::enabled(Xor(0x5a)),
        mcproto_network::PacketLimits::default(),
    );
    let frame = encoder.encode(&packet()).unwrap();
    let mut reader = PacketReader::new(
        frame.as_slice(),
        CompressionMode::enabled(0, MarkerCompression),
        DecryptionMode::enabled(Xor(0x5a)),
        PacketReadLimits::default(),
    );

    let payload = reader.read_packet().unwrap();

    assert_eq!(payload.packet_id.get(), 0x2a);
    assert_eq!(payload.body(), [0xac, 0x02]);
}

#[test]
fn reads_an_uncompressed_payload_inside_a_compressed_frame() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::enabled(1024, MarkerCompression),
        EncryptionMode::disabled(),
        mcproto_network::PacketLimits::default(),
    );
    let frame = encoder.encode(&packet()).unwrap();
    let mut reader = PacketReader::new(
        frame.as_slice(),
        CompressionMode::enabled(1024, MarkerCompression),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    );

    let payload = reader.read_packet().unwrap();

    assert_eq!(payload.packet_id.get(), 0x2a);
    assert_eq!(payload.body(), [0xac, 0x02]);
}

#[test]
fn clean_input_end_is_reported_as_closed() {
    let mut reader = PacketReader::new(
        [].as_slice(),
        CompressionMode::disabled(),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    );

    assert!(matches!(reader.read_packet(), Err(NetworkError::Closed)));
}

#[test]
fn truncated_frame_reports_expected_and_received_lengths() {
    let mut reader = PacketReader::new(
        [0x05, 0x01].as_slice(),
        CompressionMode::disabled(),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    );

    assert!(matches!(
        reader.read_packet(),
        Err(NetworkError::TruncatedFrame {
            expected: 5,
            received: 1,
        })
    ));
}

#[test]
fn oversized_declared_uncompressed_length_is_rejected_before_decompression() {
    let mut reader = PacketReader::new(
        [0x02, 0x05, 0x00].as_slice(),
        CompressionMode::enabled(0, MarkerCompression),
        DecryptionMode::disabled(),
        PacketReadLimits {
            max_frame_length: 1024,
            max_uncompressed_length: 4,
        },
    );

    assert!(matches!(
        reader.read_packet(),
        Err(NetworkError::Compression(
            CompressionError::OutputTooLarge {
                actual: 5,
                maximum: 4,
            }
        ))
    ));
}

#[test]
fn negative_packet_id_is_rejected_as_a_frame_error() {
    let mut reader = PacketReader::new(
        [0x05, 0xff, 0xff, 0xff, 0xff, 0x0f].as_slice(),
        CompressionMode::disabled(),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    );

    assert!(matches!(
        reader.read_packet(),
        Err(NetworkError::Frame(FrameError::InvalidPacketId {
            value: -1
        }))
    ));
}
