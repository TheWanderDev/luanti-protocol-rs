# Packet Writer

[`crate::protocol::writer::PacketWriter`] serializes primitive fields into a `BytesMut` buffer and returns immutable `Bytes` through `into_inner`.

## Construction and output

- `PacketWriter::new()` starts with a 64-byte capacity.
- `PacketWriter::with_capacity(capacity)` lets the caller select an initial capacity.
- `into_inner(self)` freezes and returns the accumulated bytes.

## Supported writes

The writer provides `write_u8`, `write_u16`, `write_u32`, `write_u64`, `write_i16`, `write_i32`, and `write_f32`. Integer values are written in big-endian order. `write_f32` writes the IEEE-754 representation through the byte buffer API.

`write_string16` and `write_string32` prefix UTF-8 bytes with a `u16` or `u32` byte length. `write_binary_string16` prefixes arbitrary bytes with a `u16` length. `write_wide_string16` writes a count of UTF-16 code units followed by UTF-16BE data. `write_bytes` appends an unprefixed byte slice.

## Length validation

Length-prefixed methods convert the encoded length to the corresponding integer width before writing. Values that do not fit return [`crate::ProtocolError::InvalidStringLength`]. Callers should propagate this error rather than truncating lengths.
