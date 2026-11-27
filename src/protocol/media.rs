use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::compression::{ZSTD_PROTOCOL_VERSION, decompress_limited};
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use bytes::Bytes;
const MAX_MEDIA_FILE_SIZE: usize = 16_700_000;
const MAX_ANNOUNCE_SIZE: usize = 16_700_000;
const MEDIA_NAME_ALLOWED_CHARS: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnouncedMediaFile {
    pub name: String,
    pub sha1: [u8; 20],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnounceMedia {
    pub files: Vec<AnnouncedMediaFile>,
    pub remote_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaFile {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaBunch {
    pub total_bunches: u16,
    pub index: u16,
    pub files: Vec<MediaFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPushUpdate {
    pub raw_hash: [u8; 20],
    pub filename: String,
    pub cached: bool,
    pub callback_token: Option<u32>,
    pub file_data: Option<Vec<u8>>,
}

fn decode_media_compressed(
    compressed: &[u8],
    max_size: usize,
    protocol_version: u16,
) -> Result<Vec<u8>, ProtocolError> {
    decompress_limited(compressed, max_size, protocol_version).map_err(ProtocolError::InvalidMedia)
}
fn decode_utf8(value: &[u8]) -> Result<String, ProtocolError> {
    String::from_utf8(value.to_vec())
        .map_err(|_| ProtocolError::InvalidMedia("invalid UTF-8 string"))
}

fn validate_media_name(name: &str) -> Result<(), ProtocolError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| MEDIA_NAME_ALLOWED_CHARS.contains(&byte))
    {
        return Err(ProtocolError::InvalidMedia("invalid media filename"));
    }

    Ok(())
}

pub fn decode_media_push(
    data: &[u8],
    protocol_version: u16,
) -> Result<MediaPushUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let raw_hash: [u8; 20] = reader
        .read_string16()?
        .try_into()
        .map_err(|_| ProtocolError::InvalidMedia("invalid media SHA-1 digest"))?;
    let filename = decode_utf8(reader.read_string16()?)?;
    validate_media_name(&filename)?;
    let cached = reader.read_u8()? != 0;

    let (callback_token, file_data) = if protocol_version >= 40 {
        (Some(reader.read_u32()?), None)
    } else {
        let file_data = reader.read_string32()?.to_vec();
        if file_data.is_empty() || file_data.len() > MAX_MEDIA_FILE_SIZE {
            return Err(ProtocolError::InvalidMedia(
                "invalid legacy media payload size",
            ));
        }
        (None, Some(file_data))
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMedia(
            "trailing bytes after media push",
        ));
    }

    Ok(MediaPushUpdate {
        raw_hash,
        filename,
        cached,
        callback_token,
        file_data,
    })
}

pub fn decode_announce_media(
    data: &[u8],
    protocol_version: u16,
) -> Result<AnnounceMedia, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let mut files = Vec::new();

    if protocol_version >= ZSTD_PROTOCOL_VERSION {
        let compressed_length = usize::try_from(reader.read_u32()?)
            .map_err(|_| ProtocolError::InvalidMedia("invalid announce length"))?;

        if compressed_length > MAX_ANNOUNCE_SIZE {
            return Err(ProtocolError::InvalidMedia(
                "announce data exceeds size limit",
            ));
        }

        let compressed_names = reader.read_bytes(compressed_length)?;
        let names_data =
            decode_media_compressed(compressed_names, MAX_ANNOUNCE_SIZE, protocol_version)?;

        let mut names_reader = PacketReader::new(&names_data);

        let file_count = usize::try_from(names_reader.read_u32()?)
            .map_err(|_| ProtocolError::InvalidMedia("invalid media file count"))?;

        if file_count > names_reader.remaining() / 3 {
            return Err(ProtocolError::InvalidMedia(
                "media filename array is truncated",
            ));
        }

        let required_digest_bytes = file_count
            .checked_mul(20)
            .ok_or(ProtocolError::InvalidMedia("invalid media file count"))?;
        let available_digest_bytes = reader.remaining().saturating_sub(2);

        if required_digest_bytes > available_digest_bytes {
            return Err(ProtocolError::InvalidMedia(
                "media digest array is truncated",
            ));
        }

        let mut name_lengths = Vec::with_capacity(file_count);

        for _ in 0..file_count {
            name_lengths.push(usize::from(names_reader.read_u16()?));
        }

        let mut names = Vec::with_capacity(file_count);

        for length in name_lengths {
            let name = decode_utf8(names_reader.read_bytes(length)?)?;

            validate_media_name(&name)?;

            names.push(name);
        }

        if !names_reader.is_empty() {
            return Err(ProtocolError::InvalidMedia(
                "trailing bytes in media filename array",
            ));
        }

        for name in names {
            let digest: [u8; 20] = reader
                .read_bytes(20)?
                .try_into()
                .map_err(|_| ProtocolError::InvalidMedia("invalid SHA-1 digest"))?;

            files.push(AnnouncedMediaFile { name, sha1: digest });
        }
    } else {
        let file_count = usize::from(reader.read_u16()?);

        for _ in 0..file_count {
            let name = decode_utf8(reader.read_string16()?)?;

            validate_media_name(&name)?;

            let encoded_hash = reader.read_string16()?;
            let decoded_hash = STANDARD
                .decode(encoded_hash)
                .map_err(|_| ProtocolError::InvalidMedia("invalid Base64 SHA-1 digest"))?;

            let sha1: [u8; 20] = decoded_hash
                .try_into()
                .map_err(|_| ProtocolError::InvalidMedia("invalid SHA-1 digest length"))?;

            files.push(AnnouncedMediaFile { name, sha1 });
        }
    }

    let remote_url = decode_utf8(reader.read_string16()?)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMedia(
            "trailing bytes in announce packet",
        ));
    }

    Ok(AnnounceMedia { files, remote_url })
}

pub fn decode_media_bunch(data: &[u8], protocol_version: u16) -> Result<MediaBunch, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let total_bunches = reader.read_u16()?;
    let index = reader.read_u16()?;
    let file_count = usize::try_from(reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidMedia("invalid media file count"))?;

    if total_bunches == 0 || index >= total_bunches {
        return Err(ProtocolError::InvalidMedia("invalid media bunch index"));
    }

    if file_count > reader.remaining() / 8 {
        return Err(ProtocolError::InvalidMedia(
            "media file count exceeds packet size",
        ));
    }

    let mut files = Vec::with_capacity(file_count);

    for _ in 0..file_count {
        let name = decode_utf8(reader.read_string16()?)?;

        validate_media_name(&name)?;

        let media_length = usize::try_from(reader.read_u32()?)
            .map_err(|_| ProtocolError::InvalidMedia("invalid media length"))?;
        let max_media_length = if protocol_version >= ZSTD_PROTOCOL_VERSION {
            zstd::zstd_safe::compress_bound(MAX_MEDIA_FILE_SIZE)
        } else {
            MAX_MEDIA_FILE_SIZE
        };

        if media_length == 0 || media_length > max_media_length {
            return Err(ProtocolError::InvalidMedia("invalid media payload size"));
        }

        let media_data = reader.read_bytes(media_length)?;
        let file_data = if protocol_version >= ZSTD_PROTOCOL_VERSION {
            decode_media_compressed(media_data, MAX_MEDIA_FILE_SIZE, protocol_version)?
        } else {
            media_data.to_vec()
        };
        files.push(MediaFile {
            name,
            data: file_data,
        });
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMedia("trailing bytes in media bunch"));
    }

    Ok(MediaBunch {
        total_bunches,
        index,
        files,
    })
}

pub fn encode_request_media(filenames: &[String]) -> Result<Bytes, ProtocolError> {
    let count = u16::try_from(filenames.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::RequestMedia));
    writer.write_u16(count);

    for filename in filenames {
        if filename.is_empty() {
            return Err(ProtocolError::InvalidMedia("empty requested filename"));
        }

        writer.write_string16(filename)?;
    }

    Ok(writer.into_inner())
}

pub fn encode_have_media(tokens: &[u32]) -> Result<Bytes, ProtocolError> {
    let count = u8::try_from(tokens.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

    let mut writer = PacketWriter::with_capacity(3 + tokens.len() * 4);
    writer.write_u16(u16::from(ToServerCommand::HaveMedia));
    writer.write_u8(count);

    for token in tokens {
        writer.write_u32(*token);
    }

    Ok(writer.into_inner())
}
