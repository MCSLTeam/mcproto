//! Integration tests for the protocol `Fixed BitSet (n)` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{TypeCodec, basic::FixedBitSet};

#[test]
fn zero_bit_set_encodes_no_bytes() {
    let bits = FixedBitSet::<0>(Vec::new());

    let mut encoded = Vec::new();
    bits.encode(&mut encoded).unwrap();
    assert!(encoded.is_empty());

    let mut input = encoded.as_slice();
    assert_eq!(FixedBitSet::<0>::decode(&mut input).unwrap(), bits);
}

#[test]
fn nine_bits_use_two_bytes_with_documented_bit_order() {
    let bits = FixedBitSet::<9>(vec![0b1000_0001, 0b0000_0001]);

    let mut encoded = Vec::new();
    bits.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0b1000_0001, 0b0000_0001]);

    let mut input = encoded.as_slice();
    assert_eq!(FixedBitSet::<9>::decode(&mut input).unwrap(), bits);
    assert!(bits.contains(0));
    assert!(bits.contains(7));
    assert!(bits.contains(8));
    assert!(!bits.contains(1));
    assert!(!bits.contains(9));
}

#[test]
fn bytes_cover_multiple_boundaries() {
    let bits = FixedBitSet::<65>(vec![0x01, 0x80, 0, 0, 0, 0, 0, 0, 0x01]);

    assert!(bits.contains(0));
    assert!(bits.contains(15));
    assert!(bits.contains(64));
    assert!(!bits.contains(63));
}

#[test]
fn encoding_rejects_incorrect_byte_length() {
    let error = FixedBitSet::<9>(vec![0])
        .encode(&mut Vec::new())
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidFixedBitSetLength {
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(error.codec(), CodecKind::FixedBitSet);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn padding_bits_are_rejected_when_encoding_or_decoding() {
    let error = FixedBitSet::<9>(vec![0, 0b0000_0010])
        .encode(&mut Vec::new())
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ValueOutOfRange {
            terminal_byte: 0b0000_0010,
            allowed_mask: 0b0000_0001,
        })
    );
    assert_eq!(error.operation(), CodecOperation::Write);

    let mut input = [0, 0b1111_1110].as_slice();
    let error = FixedBitSet::<9>::decode(&mut input).unwrap_err();
    assert_eq!(error.codec(), CodecKind::FixedBitSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 2);
}

#[test]
fn truncated_data_reports_eof_with_exact_progress() {
    let mut input = [0x01].as_slice();
    let error = FixedBitSet::<9>::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::FixedBitSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}
