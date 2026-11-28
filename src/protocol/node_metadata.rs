use ahash::AHashSet;

use glam::I16Vec3;

use crate::errors::ProtocolError;
use crate::protocol::compression::decompress_zlib_frame;
use crate::protocol::reader::PacketReader;

const MAX_METADATA_STRING_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMetadataField {
    pub key: String,
    pub value: String,
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMetadata {
    pub position: I16Vec3,
    pub fields: Vec<NodeMetadataField>,
    pub inventory: String,
}

impl NodeMetadata {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
            && (self.inventory.is_empty()
                || self.inventory == "EndInventory\n"
                || self.inventory == "EndInventory"
                || self.inventory == "end\n"
                || self.inventory == "end")
    }
}

fn read_metadata_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidMapBlock("metadata string is not UTF-8"))
}

fn read_metadata_string32(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let length = reader.read_len32()?;

    if length > MAX_METADATA_STRING_SIZE {
        return Err(ProtocolError::InvalidMapBlock(
            "metadata string exceeds size limit",
        ));
    }

    let bytes = reader.read_bytes(length)?;
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidMapBlock("metadata string is not UTF-8"))
}

fn read_metadata_inventory(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let mut inventory = Vec::new();

    loop {
        let Some(line) = reader.read_line()? else {
            return Err(ProtocolError::InvalidMapBlock(
                "metadata inventory terminator is missing",
            ));
        };

        inventory.extend_from_slice(line);

        if matches!(
            line,
            b"EndInventory\n" | b"EndInventory" | b"end\n" | b"end"
        ) {
            break;
        }
    }

    String::from_utf8(inventory)
        .map_err(|_| ProtocolError::InvalidMapBlock("metadata inventory is not UTF-8"))
}

fn decode_metadata_entry(
    reader: &mut PacketReader<'_>,
    version: u8,
    position: I16Vec3,
) -> Result<NodeMetadata, ProtocolError> {
    let field_count =
        usize::try_from(reader.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)?;

    let minimum_field_size = if version >= 2 { 7 } else { 6 };
    if field_count > reader.remaining() / minimum_field_size {
        return Err(ProtocolError::InvalidMapBlock(
            "invalid metadata field count",
        ));
    }

    let mut fields = Vec::with_capacity(field_count);

    for _ in 0..field_count {
        let key = read_metadata_string16(reader)?;
        let value = read_metadata_string32(reader)?;

        let is_private = if version >= 2 {
            match reader.read_u8()? {
                0 => false,
                1 => true,
                _ => {
                    return Err(ProtocolError::InvalidMapBlock(
                        "invalid metadata privacy flag",
                    ));
                }
            }
        } else {
            false
        };

        fields.push(NodeMetadataField {
            key,
            value,
            is_private,
        });
    }

    let inventory = read_metadata_inventory(reader)?;

    Ok(NodeMetadata {
        position,
        fields,
        inventory,
    })
}

pub fn decode_node_metadata_list(
    data: &[u8],
    world_positions: bool,
) -> Result<Vec<NodeMetadata>, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version == 0 {
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidMapBlock(
                "unexpected bytes after empty metadata list",
            ));
        }

        return Ok(Vec::new());
    }

    if !(1..=2).contains(&version) {
        return Err(ProtocolError::InvalidMapBlock(
            "unsupported node metadata version",
        ));
    }

    let count = usize::from(reader.read_u16()?);
    let position_size = if world_positions { 6 } else { 2 };
    let minimum_entry_size = position_size + 4 + "end".len();

    if count > reader.remaining() / minimum_entry_size {
        return Err(ProtocolError::InvalidMapBlock(
            "metadata entry count exceeds packet size",
        ));
    }

    let mut metadata = Vec::with_capacity(count);
    let mut positions = AHashSet::with_capacity(count);

    for _ in 0..count {
        let position = if world_positions {
            I16Vec3::new(reader.read_i16()?, reader.read_i16()?, reader.read_i16()?)
        } else {
            let packed = reader.read_u16()?;

            if usize::from(packed) >= 16 * 16 * 16 {
                return Err(ProtocolError::InvalidMapBlock(
                    "invalid local metadata position",
                ));
            }

            let x = i16::try_from(packed & 0x0F)
                .map_err(|_| ProtocolError::InvalidMapBlock("invalid metadata position"))?;
            let y = i16::try_from((packed >> 4) & 0x0F)
                .map_err(|_| ProtocolError::InvalidMapBlock("invalid metadata position"))?;
            let z = i16::try_from(packed >> 8)
                .map_err(|_| ProtocolError::InvalidMapBlock("invalid metadata position"))?;

            I16Vec3::new(x, y, z)
        };

        if !positions.insert(position) {
            return Err(ProtocolError::InvalidMapBlock(
                "duplicate node metadata position",
            ));
        }

        metadata.push(decode_metadata_entry(&mut reader, version, position)?);
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMapBlock(
            "unexpected bytes after node metadata list",
        ));
    }

    Ok(metadata)
}

pub(crate) fn decode_node_metadata_changed(
    payload: &[u8],
) -> Result<Vec<NodeMetadata>, ProtocolError> {
    let mut reader = PacketReader::new(payload);

    let compressed_length =
        usize::try_from(reader.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)?;

    if compressed_length != payload.len().saturating_sub(4) {
        return Err(ProtocolError::InvalidMapBlock(
            "invalid node metadata compressed length",
        ));
    }

    let compressed = reader.read_bytes(compressed_length)?;

    let (decompressed, consumed) =
        decompress_zlib_frame(compressed).map_err(ProtocolError::InvalidMapBlock)?;

    if consumed != compressed.len() {
        return Err(ProtocolError::InvalidMapBlock(
            "unexpected bytes after compressed node metadata",
        ));
    }

    decode_node_metadata_list(&decompressed, true)
}
