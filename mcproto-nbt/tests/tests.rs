#[cfg(test)]
mod tests {
    use mcproto_nbt::NbtReader;
    use mcproto_nbt::TagKind;
    // 逻辑测试
    #[test]
    // 所有类型的读取
    fn reader_all_types() {
        let buf: &[u8] = &[
            0x7f, // u8 = 127
            0xfe, // i8 = -2
            0x12, 0x34, // i16 = 0x1234
            0xab, 0xcd, // u16 = 0xabcd
            0x01, 0x23, 0x45, 0x67, // i32 = 0x01234567
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, // i64
            0x3f, 0xc0, 0x00, 0x00, // f32 = 1.5
            0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f64 = 2.5
            0x0a, // TagKind::Compound
            0x00, 0x05, b'h', b'e', b'l', b'l', b'o', // string = "hello"
        ];

        let mut reader = NbtReader::new(buf);

        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), buf.len());

        assert_eq!(reader.read_u8().unwrap(), 0x7f);
        assert_eq!(reader.read_i8().unwrap(), -2);
        assert_eq!(reader.read_i16().unwrap(), 0x1234);
        assert_eq!(reader.read_u16().unwrap(), 0xabcd);
        assert_eq!(reader.read_i32().unwrap(), 0x0123_4567);
        assert_eq!(reader.read_i64().unwrap(), 0x0123_4567_89ab_cdef);
        assert_eq!(reader.read_f32().unwrap(), 1.5);
        assert_eq!(reader.read_f64().unwrap(), 2.5);
        assert_eq!(reader.read_tag_kind().unwrap(), TagKind::Compound);
        assert_eq!(reader.read_string().unwrap(), "hello");

        assert!(reader.is_empty());
        assert_eq!(reader.remaining(), 0);
        assert_eq!(reader.position(), buf.len());
    }
    #[test]
    // peek测试，也就是只读不消耗
    fn reader_peek() {
        let buf: &[u8] = &[0x0a, 0x00, 0x03, b'n', b'b', b't'];
        let mut reader = NbtReader::new(buf);

        assert_eq!(reader.position(), 0);
        assert_eq!(reader.peek_u8().unwrap(), 0x0a);
        assert_eq!(reader.peek_tag_kind().unwrap(), TagKind::Compound);
        assert_eq!(reader.peek_string().unwrap_err().to_string(), "Unexpected end of buffer at offset 2: need 2560 bytes, remaining 4");
        assert_eq!(reader.position(),  0);

        assert_eq!(reader.read_tag_kind().unwrap(), TagKind::Compound);
        assert_eq!(reader.peek_string().unwrap(), "nbt");
        assert_eq!(reader.position(), 1);
        assert_eq!(reader.read_string().unwrap(), "nbt");
        assert!(reader.is_empty());
    }
}

