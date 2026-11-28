use std::io::{Cursor, Read};

use flate2::read::ZlibDecoder;

pub const ZSTD_PROTOCOL_VERSION: u16 = 48;

fn decompression_limit(max_size: usize) -> Result<u64, &'static str> {
    u64::try_from(max_size)
        .map_err(|_| "invalid decompression limit")?
        .checked_add(1)
        .ok_or("invalid decompression limit")
}

pub fn decompress_zstd_frame(compressed: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut decoder = zstd::stream::read::Decoder::new(Cursor::new(compressed))
        .map_err(|_| "invalid zstd frame")?
        .single_frame();
    let mut output = Vec::new();

    decoder
        .read_to_end(&mut output)
        .map_err(|_| "zstd decompression failed")?;

    decoder
        .finish_frame()
        .map_err(|_| "zstd frame is incomplete")?;
    let reader = decoder.finish();
    let buffered =
        u64::try_from(reader.buffer().len()).map_err(|_| "invalid zstd buffer length")?;
    let input_length = u64::try_from(compressed.len()).map_err(|_| "invalid zstd input length")?;
    let consumed = reader.get_ref().position().saturating_sub(buffered);
    if consumed != input_length {
        return Err("unexpected bytes after zstd frame");
    }

    Ok(output)
}

pub fn decompress_zlib_frame(compressed: &[u8]) -> Result<(Vec<u8>, usize), &'static str> {
    let mut decoder = ZlibDecoder::new(Cursor::new(compressed));
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|_| "zlib decompression failed")?;

    let consumed = usize::try_from(decoder.total_in()).map_err(|_| "invalid zlib frame length")?;
    if consumed == 0 || consumed > compressed.len() {
        return Err("invalid zlib frame length");
    }

    Ok((output, consumed))
}

pub fn decompress_zstd_limited(
    compressed: &[u8],
    max_size: usize,
) -> Result<Vec<u8>, &'static str> {
    let limit = decompression_limit(max_size)?;

    let mut decoder = zstd::stream::read::Decoder::new(Cursor::new(compressed))
        .map_err(|_| "invalid zstd frame")?
        .single_frame();
    decoder
        .window_log_max(23)
        .map_err(|_| "zstd window exceeds size limit")?;

    let mut output = Vec::new();

    (&mut decoder)
        .take(limit)
        .read_to_end(&mut output)
        .map_err(|_| "zstd decompression failed")?;

    if output.len() > max_size {
        return Err("decompressed data exceeds size limit");
    }

    decoder
        .finish_frame()
        .map_err(|_| "zstd frame is incomplete")?;
    let reader = decoder.finish();
    let buffered =
        u64::try_from(reader.buffer().len()).map_err(|_| "invalid zstd buffer length")?;
    let input_length = u64::try_from(compressed.len()).map_err(|_| "invalid zstd input length")?;
    let consumed = reader.get_ref().position().saturating_sub(buffered);
    if consumed != input_length {
        return Err("unexpected bytes after zstd frame");
    }

    Ok(output)
}

pub fn decompress_zlib_frame_limited(
    compressed: &[u8],
    max_size: usize,
) -> Result<(Vec<u8>, usize), &'static str> {
    let limit = decompression_limit(max_size)?;

    let mut decoder = ZlibDecoder::new(Cursor::new(compressed));
    let mut output = Vec::new();

    decoder
        .by_ref()
        .take(limit)
        .read_to_end(&mut output)
        .map_err(|_| "zlib decompression failed")?;

    if output.len() > max_size {
        return Err("decompressed data exceeds size limit");
    }

    let consumed = usize::try_from(decoder.total_in()).map_err(|_| "invalid zlib frame length")?;

    if consumed == 0 || consumed > compressed.len() {
        return Err("invalid zlib frame length");
    }

    Ok((output, consumed))
}

pub fn decompress_limited(
    compressed: &[u8],
    max_size: usize,
    protocol_version: u16,
) -> Result<Vec<u8>, &'static str> {
    if protocol_version >= ZSTD_PROTOCOL_VERSION {
        decompress_zstd_limited(compressed, max_size)
    } else {
        let (output, consumed) = decompress_zlib_frame_limited(compressed, max_size)?;
        if consumed != compressed.len() {
            return Err("unexpected bytes after zlib frame");
        }
        Ok(output)
    }
}
