//! Integration tests for the protocol `Angle` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation};
use mcproto_types::{TypeCodec, basic::Angle};

#[test]
fn byte_values_roundtrip() {
    for value in [0, 64, 128, 255] {
        let mut encoded = Vec::new();
        Angle(value).encode(&mut encoded).unwrap();
        assert_eq!(encoded, [value]);

        let mut input = encoded.as_slice();
        assert_eq!(Angle::decode(&mut input).unwrap(), Angle(value));
        assert!(input.is_empty());
    }
}

#[test]
fn converts_to_degrees() {
    assert_eq!(Angle(0).to_degrees(), 0.0);
    assert_eq!(Angle(64).to_degrees(), 90.0);
    assert_eq!(Angle(128).to_degrees(), 180.0);
    assert!((Angle(255).to_degrees() - 358.59375).abs() < 1e-12);
}

#[test]
fn converts_to_radians() {
    assert_eq!(Angle(0).to_radians(), 0.0);
    assert!((Angle(64).to_radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    assert!((Angle(128).to_radians() - std::f64::consts::PI).abs() < 1e-12);
    assert!((Angle(255).to_radians() - 255.0 * std::f64::consts::TAU / 256.0).abs() < 1e-12);
}

#[test]
fn empty_input_reports_eof_with_angle_codec() {
    let mut input = [].as_slice();
    let error = Angle::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Angle);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 0);
}
