//! Integration tests for the `TypeStructCodec` derive.

use mcproto_codec::error::{CodecErrorKind, CodecKind};
use mcproto_types::{Int, TypeCodec, TypeStructCodec, UnsignedByte};

#[derive(Debug, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
struct Named {
    first: UnsignedByte,
    second: Int,
}

#[derive(Debug, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
struct Generic<T>(UnsignedByte, T);

#[derive(Debug, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
struct Unit;

#[test]
fn named_fields_are_encoded_in_declaration_order() {
    let value = Named {
        first: UnsignedByte(0xaa),
        second: Int(0x0102_0304),
    };
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0xaa, 0x01, 0x02, 0x03, 0x04]);

    let mut input = encoded.as_slice();
    assert_eq!(Named::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn tuple_generics_and_unit_structs_are_supported() {
    let value = Generic(UnsignedByte(7), Int(9));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();
    assert_eq!(Generic::<Int>::decode(&mut input).unwrap(), value);

    let mut encoded = Vec::new();
    Unit.encode(&mut encoded).unwrap();
    assert!(encoded.is_empty());
    assert_eq!(Unit::decode(&mut encoded.as_slice()).unwrap(), Unit);
}

#[test]
fn field_errors_keep_the_struct_context() {
    let mut input = [0xaa, 0x01].as_slice();
    let error = Named::decode(&mut input).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Int);
    assert_eq!(error.contexts(), &[CodecKind::TypeStruct]);
    assert_eq!(error.bytes_processed(), 1);
}
