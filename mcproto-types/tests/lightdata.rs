//! Protocol tests for Light Data.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    BitSet, LIGHT_ARRAY_LENGTH, LIGHT_VALUES_PER_ARRAY, LightArray, LightData, PrefixedArray,
    TypeCodec,
};

#[test]
fn empty_light_data_is_six_zero_length_prefixes() {
    let value = LightData::default();
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0, 0, 0, 0, 0, 0]);

    let mut input = encoded.as_slice();
    assert_eq!(LightData::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn light_array_uses_an_exact_2048_byte_prefix_and_nibbles() {
    let mut bytes = [0; LIGHT_ARRAY_LENGTH];
    bytes[0] = 0xa3;
    bytes[LIGHT_ARRAY_LENGTH - 1] = 0xf1;
    let value = LightArray(bytes);

    assert_eq!(value.light_level(0), Some(3));
    assert_eq!(value.light_level(1), Some(10));
    assert_eq!(value.light_level(LIGHT_VALUES_PER_ARRAY - 2), Some(1));
    assert_eq!(value.light_level(LIGHT_VALUES_PER_ARRAY - 1), Some(15));
    assert_eq!(value.light_level(LIGHT_VALUES_PER_ARRAY), None);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(&encoded[..2], [0x80, 0x10]);
    assert_eq!(encoded.len(), LIGHT_ARRAY_LENGTH + 2);

    let mut input = encoded.as_slice();
    assert_eq!(LightArray::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn populated_light_data_roundtrips_in_mask_bit_order() {
    let mut first_sky = LightArray::default();
    first_sky.0[0] = 0x11;
    let mut second_sky = LightArray::default();
    second_sky.0[0] = 0x22;
    let mut block = LightArray::default();
    block.0[0] = 0x33;

    let value = LightData {
        sky_light_mask: BitSet(vec![0b0101]),
        block_light_mask: BitSet(vec![0b0010]),
        empty_sky_light_mask: BitSet::default(),
        empty_block_light_mask: BitSet::default(),
        sky_light_arrays: PrefixedArray(vec![first_sky, second_sky]),
        block_light_arrays: PrefixedArray(vec![block]),
    };

    assert_eq!(value.expected_sky_light_array_count(), 2);
    assert_eq!(value.expected_block_light_array_count(), 1);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(
        &encoded[..20],
        [
            1, 0, 0, 0, 0, 0, 0, 0, 5, // Sky mask
            1, 0, 0, 0, 0, 0, 0, 0, 2, // Block mask
            0, 0, // Empty masks
        ]
    );
    assert_eq!(encoded[20], 2);
    assert_eq!(&encoded[21..23], [0x80, 0x10]);

    let mut input = encoded.as_slice();
    assert_eq!(LightData::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn invalid_inner_light_array_length_is_rejected() {
    let encoded = [0xff, 0x0f]; // 2047
    let error = LightArray::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ArrayLengthMismatch {
            expected: LIGHT_ARRAY_LENGTH,
            actual: 2047,
        })
    );
    assert_eq!(error.codec(), CodecKind::LightArray);
}

#[test]
fn mask_and_array_count_mismatch_is_rejected_on_write() {
    let value = LightData {
        sky_light_mask: BitSet(vec![1]),
        ..LightData::default()
    };

    let error = value.encode(&mut Vec::new()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ArrayLengthMismatch {
            expected: 1,
            actual: 0,
        })
    );
    assert_eq!(error.codec(), CodecKind::LightData);
    assert_eq!(error.operation(), CodecOperation::Write);
}

#[test]
fn mask_and_array_count_mismatch_is_rejected_on_read() {
    let encoded = [
        1, 0, 0, 0, 0, 0, 0, 0, 1, // Sky mask with bit zero set
        0, // Block mask
        0, // Empty sky mask
        0, // Empty block mask
        0, // No sky arrays
        0, // No block arrays
    ];

    let error = LightData::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ArrayLengthMismatch {
            expected: 1,
            actual: 0,
        })
    );
    assert_eq!(error.codec(), CodecKind::LightData);
    assert_eq!(error.operation(), CodecOperation::Read);
}
