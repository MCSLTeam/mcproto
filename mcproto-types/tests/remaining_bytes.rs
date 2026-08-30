//! Tests for payloads delimited by their enclosing value.

use mcproto_types::{RemainingBytes, TypeCodec};

#[test]
fn remaining_bytes_have_no_length_prefix() {
    let value = RemainingBytes::new(vec![0x00, 0x80, 0xff]);
    let mut encoded = Vec::new();

    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00, 0x80, 0xff]);

    let mut input = encoded.as_slice();
    assert_eq!(RemainingBytes::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}
