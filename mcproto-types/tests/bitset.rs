//! Integration tests for the protocol `BitSet` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{TypeCodec, basic::BitSet};

#[test]
fn empty_bitset_roundtrips() {
    let bits = BitSet(Vec::new());

    let mut encoded = Vec::new();
    bits.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = encoded.as_slice();
    assert_eq!(BitSet::decode(&mut input).unwrap(), bits);
    assert!(input.is_empty());
}

#[test]
fn single_word_roundtrips_with_expected_bytes() {
    let bits = BitSet(vec![0x05]);

    let mut encoded = Vec::new();
    bits.encode(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05]
    );

    let mut input = encoded.as_slice();
    assert_eq!(BitSet::decode(&mut input).unwrap(), bits);
    assert!(input.is_empty());
}

#[test]
fn contains_uses_the_documented_bit_order() {
    let bits = BitSet(vec![0b0000_0101]);

    assert!(bits.contains(0));
    assert!(!bits.contains(1));
    assert!(bits.contains(2));
    assert!(!bits.contains(3));
    assert!(!bits.contains(64));
}

#[test]
fn multiple_words_roundtrip() {
    let words = vec![u64::MAX, 0, 1 << 63, 0x0123_4567_89ab_cdef];
    let bits = BitSet(words.clone());

    let mut encoded = Vec::new();
    bits.encode(&mut encoded).unwrap();

    let mut input = encoded.as_slice();
    assert_eq!(BitSet::decode(&mut input).unwrap(), BitSet(words));
    assert!(input.is_empty());
}

#[test]
fn negative_length_is_rejected() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x0f].as_slice();
    let error = BitSet::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::NegativeLength { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::BitSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn truncated_long_data_reports_eof_with_exact_progress() {
    let mut input = [0x01].as_slice();
    let error = BitSet::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::BitSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}
