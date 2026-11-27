use std::io::Write;

use flate2::{Compression, write::ZlibEncoder};
use luanti_protocol_rs::protocol::compression::{
    decompress_limited, decompress_zlib_frame_limited, decompress_zstd_limited,
};

fn zlib_frame(input: &[u8]) -> Option<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(input).is_err() {
        return None;
    }
    encoder.finish().ok()
}

#[test]
fn zstd_decompression_round_trips_with_size_limit() {
    let input = b"protocol payload repeated protocol payload";
    let compressed_result = zstd::encode_all(input.as_slice(), 3);
    assert!(compressed_result.is_ok());
    let compressed = compressed_result.unwrap_or_default();

    assert_eq!(
        decompress_zstd_limited(&compressed, input.len()).as_deref(),
        Ok(input.as_slice())
    );
    assert!(decompress_zstd_limited(&compressed, input.len() - 1).is_err());
}

#[test]
fn zlib_decompression_reports_frame_length_and_enforces_limit() {
    let input = b"legacy protocol payload";
    let compressed_result = zlib_frame(input);
    assert!(compressed_result.is_some());
    let compressed = compressed_result.unwrap_or_default();

    let decoded = decompress_zlib_frame_limited(&compressed, input.len());
    assert_eq!(
        decoded.as_ref().map(|value| value.0.as_slice()),
        Ok(input.as_slice())
    );
    assert_eq!(decoded.map(|value| value.1), Ok(compressed.len()));
    assert!(decompress_zlib_frame_limited(&compressed, input.len() - 1).is_err());
}

#[test]
fn protocol_version_selects_compression_format() {
    let input = b"versioned compression payload";
    let zstd_result = zstd::encode_all(input.as_slice(), 1);
    assert!(zstd_result.is_ok());
    let zstd = zstd_result.unwrap_or_default();
    let zlib_result = zlib_frame(input);
    assert!(zlib_result.is_some());
    let zlib = zlib_result.unwrap_or_default();

    assert_eq!(
        decompress_limited(&zlib, input.len(), 47).as_deref(),
        Ok(input.as_slice())
    );
    assert_eq!(
        decompress_limited(&zstd, input.len(), 48).as_deref(),
        Ok(input.as_slice())
    );
}

#[test]
fn compression_rejects_invalid_frames() {
    assert!(decompress_zstd_limited(&[0xFF, 0x00], 1024).is_err());
    assert!(decompress_zlib_frame_limited(&[0xFF, 0x00], 1024).is_err());
}

#[test]
fn decompression_rejects_trailing_bytes_after_a_frame() {
    let input = b"protocol payload";

    let zstd_result = zstd::encode_all(input.as_slice(), 1);
    assert!(zstd_result.is_ok());
    let mut zstd = zstd_result.unwrap_or_default();
    zstd.push(0);
    assert!(decompress_limited(&zstd, input.len(), 48).is_err());

    let zlib_result = zlib_frame(input);
    assert!(zlib_result.is_some());
    let mut zlib = zlib_result.unwrap_or_default();
    zlib.push(0);
    assert!(decompress_limited(&zlib, input.len(), 47).is_err());
}
