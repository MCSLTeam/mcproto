//! Integration tests for contextual protocol codecs.

use mcproto_codec::error::CodecError;
use mcproto_types::{ContextualCodec, TypeCodec, basic::UnsignedByte, contextual::Context};

#[derive(Debug, PartialEq, Eq)]
struct OptionalByte(Option<UnsignedByte>);

impl ContextualCodec for OptionalByte {
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &Context,
    ) -> Result<(), CodecError> {
        if context.is_present() {
            self.0
                .as_ref()
                .expect("the test value must match its context")
                .encode(writer)?;
        }
        Ok(())
    }

    fn decode_with_context(
        reader: &mut impl std::io::Read,
        context: &Context,
    ) -> Result<Self, CodecError> {
        if context.is_present() {
            Ok(Self(Some(UnsignedByte::decode(reader)?)))
        } else {
            Ok(Self(None))
        }
    }
}

#[test]
fn context_constructors_express_presence() {
    assert_eq!(Context::new(true), Context::PRESENT);
    assert_eq!(Context::present(), Context::PRESENT);
    assert!(Context::PRESENT.is_present());

    assert_eq!(Context::new(false), Context::ABSENT);
    assert_eq!(Context::absent(), Context::ABSENT);
    assert!(!Context::ABSENT.is_present());
}

#[test]
fn present_context_encodes_and_decodes_the_value() {
    let value = OptionalByte(Some(UnsignedByte(0xab)));
    let mut encoded = Vec::new();
    value
        .encode_with_context(&mut encoded, &Context::present())
        .unwrap();
    assert_eq!(encoded, [0xab]);

    let mut input = encoded.as_slice();
    assert_eq!(
        OptionalByte::decode_with_context(&mut input, &Context::present()).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn absent_context_consumes_and_produces_no_bytes() {
    let value = OptionalByte(None);
    let mut encoded = Vec::new();
    value
        .encode_with_context(&mut encoded, &Context::absent())
        .unwrap();
    assert!(encoded.is_empty());

    let mut input = [0xab].as_slice();
    assert_eq!(
        OptionalByte::decode_with_context(&mut input, &Context::absent()).unwrap(),
        value
    );
    assert_eq!(input, [0xab]);
}
