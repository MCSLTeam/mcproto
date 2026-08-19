//! Tests for field-specific protocol string limits.

use mcproto_types::{BoundedString, PrefixedString, TypeCodec};

#[test]
fn bounded_string_roundtrips_at_its_utf16_limit() {
    let value = BoundedString::<3>::new("a😀").unwrap();
    let mut encoded = Vec::new();

    value.encode(&mut encoded).unwrap();

    let mut input = encoded.as_slice();
    assert_eq!(BoundedString::<3>::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn bounded_string_rejects_overlong_construction_and_decoding() {
    assert!(BoundedString::<3>::new("😀😀").is_err());

    let mut encoded = Vec::new();
    PrefixedString("abcd".into()).encode(&mut encoded).unwrap();
    assert!(BoundedString::<3>::decode(&mut encoded.as_slice()).is_err());
}
