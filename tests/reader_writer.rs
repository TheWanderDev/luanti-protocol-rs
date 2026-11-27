use luanti_protocol_rs::errors::ProtocolError;
use luanti_protocol_rs::protocol::reader::PacketReader;
use luanti_protocol_rs::protocol::writer::PacketWriter;

#[test]
fn integer_and_float_primitives_use_big_endian_wire_order() {
    let mut writer = PacketWriter::new();
    writer.write_u8(0xAB);
    writer.write_u16(0x1234);
    writer.write_u32(0x1234_5678);
    writer.write_u64(0x0123_4567_89AB_CDEF);
    writer.write_i32(-123_456);
    writer.write_f32(1.5);

    let bytes = writer.into_inner();
    assert_eq!(
        bytes.as_ref(),
        &[
            0xAB, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
            0xEF, 0xFF, 0xFE, 0x1D, 0xC0, 0x3F, 0xC0, 0x00, 0x00,
        ]
    );

    let mut reader = PacketReader::new(&bytes);
    assert_eq!(reader.read_u8(), Ok(0xAB));
    assert_eq!(reader.read_u16(), Ok(0x1234));
    assert_eq!(reader.read_u32(), Ok(0x1234_5678));
    assert_eq!(reader.read_u64(), Ok(0x0123_4567_89AB_CDEF));
    assert_eq!(reader.read_i32(), Ok(-123_456));
    assert_eq!(reader.read_f32(), Ok(1.5));
    assert!(reader.is_empty());
}

#[test]
fn string_encodings_round_trip_utf8_and_utf16() {
    let mut writer = PacketWriter::new();
    assert!(writer.write_string16("héllo").is_ok());
    assert!(writer.write_wide_string16("世界🌍").is_ok());
    assert!(writer.write_binary_string16(&[0, 1, 0xFF]).is_ok());

    let bytes = writer.into_inner();
    let mut reader = PacketReader::new(&bytes);
    assert_eq!(reader.read_string16(), Ok("héllo".as_bytes()));
    assert_eq!(reader.read_wide_string16(), Ok("世界🌍".to_owned()));
    assert_eq!(reader.read_string16(), Ok(&[0, 1, 0xFF][..]));
    assert!(reader.is_empty());
}

#[test]
fn reader_reports_truncation_without_advancing_cursor() {
    let mut reader = PacketReader::new(&[0x12, 0x34, 0x56]);
    assert_eq!(reader.read_u16(), Ok(0x1234));
    assert_eq!(reader.read_u32(), Err(ProtocolError::Truncated));
    assert_eq!(reader.remaining(), 1);
    assert_eq!(reader.read_u8(), Ok(0x56));
    assert!(reader.is_empty());
}

#[test]
fn reader_rejects_invalid_utf16() {
    let bytes = [0, 1, 0xD8, 0x00];
    let mut reader = PacketReader::new(&bytes);
    assert_eq!(
        reader.read_wide_string16(),
        Err(ProtocolError::InvalidWideString)
    );
}

#[test]
fn writer_rejects_strings_that_exceed_u16_length() {
    let value = "x".repeat(usize::from(u16::MAX) + 1);
    let mut writer = PacketWriter::new();
    assert_eq!(
        writer.write_string16(&value),
        Err(ProtocolError::InvalidStringLength)
    );
}
