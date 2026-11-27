# Performance Characteristics

This section records properties visible in the source; it does not publish benchmark claims.

## Borrowed parsing

[`crate::protocol::reader::PacketReader`] stores an input slice and cursor. `read_bytes`, `read_string16`, and `read_string32` return borrowed slices, avoiding an allocation for those reads. UTF-16 decoding creates an owned `String`.

## Output allocation

[`crate::protocol::writer::PacketWriter`] uses `BytesMut` and supports caller-selected initial capacity. `into_inner` freezes the buffer into `Bytes`.

## Bulk data

Map blocks, definitions, inventories, media bunches, and active-object updates allocate owned collections sized from validated counts. The decoders perform count/remaining-byte checks before collection loops in the relevant paths.

## Compression

Limited zlib and Zstandard helpers cap decoded output using the caller-provided maximum. The Zstandard limited path also caps the decoder window log. Callers should select limits from the payload's actual protocol constraints.

## Benchmarks

The crate manifest declares Criterion benchmarks for compression, map blocks, and packets. No numeric performance result is asserted here; benchmark results must be measured on the target hardware and build profile.
