pub mod basic;

pub trait TypeCodec {
    fn encode(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), mcproto_codec::error::CodecError>;
    fn decode(reader: &mut impl std::io::Read) -> Result<Self, mcproto_codec::error::CodecError>
    where
        Self: Sized;
}
