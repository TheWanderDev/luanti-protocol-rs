# Compression

Compression helpers live in [`crate::protocol::compression`]. The module decodes zlib and Zstandard frames; it does not provide compression encoders.

## Protocol selection

[`crate::protocol::compression::ZSTD_PROTOCOL_VERSION`] is the threshold used by `decompress_limited` to select the compression format by negotiated protocol version. The current source sets this threshold to 48.

## APIs

- `decompress_zstd_frame` decodes one Zstandard frame and rejects trailing input after that frame.
- `decompress_zlib_frame` decodes a zlib stream and returns both the decoded bytes and the number of compressed bytes consumed.
- `decompress_zstd_limited` and `decompress_zlib_frame_limited` cap decoded output at the caller-supplied maximum.
- `decompress_limited` selects the version-appropriate limited decoder.

The limited functions read at most one byte beyond the configured output limit to detect an oversized result. The Zstandard limited path also sets a maximum decoder window log. Errors are returned as static string messages by these helpers; payload modules may map them into a domain-specific [`crate::ProtocolError`].
