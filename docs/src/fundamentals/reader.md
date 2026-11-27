# Packet Reader

[`crate::protocol::reader::PacketReader`] is a cursor over a borrowed byte slice. Construct it with `PacketReader::new(data)` and consume fields in wire order.

## Supported reads

- `read_bytes(length)` returns a borrowed slice.
- `read_u8`, `read_u16`, `read_u32`, `read_u64`, `read_i16`, and `read_i32` read big-endian integers.
- `read_f32` interprets a big-endian `u32` bit pattern as IEEE-754 `f32`.
- `read_f1000` reads an `i32` and divides by 1000.
- `read_string16` and `read_string32` return borrowed byte slices after their length prefixes.
- `read_len32` reads a `u32` length and converts it to `usize`.
- `read_wide_string16` decodes UTF-16BE into an owned `String`.
- `read_line` returns the next slice including its newline when one exists, or the remaining bytes at end of input.
- `remaining` and `is_empty` expose cursor state.

## Bounds and errors

A read advances the cursor after the requested range is available. Out-of-range reads return [`crate::ProtocolError::Truncated`]. Length conversion failures return `InvalidStringLength`; malformed UTF-16 returns `InvalidWideString`.

`read_string16` and `read_string32` deliberately return bytes, not `String`. The payload decoder decides whether a particular field requires UTF-8, lossy UTF-8, or another interpretation.
