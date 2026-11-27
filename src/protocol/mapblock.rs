use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::compression::{decompress_zlib_frame, decompress_zstd_frame};
use crate::protocol::node_metadata::{NodeMetadata, decode_node_metadata_list};
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use bytes::Bytes;
use glam::I16Vec3;
const MAP_BLOCK_NODE_COUNT: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapNode {
    pub content_id: u16,
    pub param1: u8,
    pub param2: u8,
}

impl MapNode {
    #[must_use]
    pub const fn day_light(self) -> u8 {
        self.param1 & 0x0F
    }

    #[must_use]
    pub const fn night_light(self) -> u8 {
        self.param1 >> 4
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapBlockData {
    pub position: I16Vec3,
    pub flags: u8,
    pub lighting_complete: u16,
    pub nodes: Vec<MapNode>,
    pub metadata_data: Vec<u8>,
    pub metadata: Vec<NodeMetadata>,
    pub network_specific_version: u8,
}

pub fn decode_add_node(
    data: &[u8],
    serialization_version: u8,
) -> Result<(I16Vec3, MapNode, bool), ProtocolError> {
    let mut reader = PacketReader::new(data);

    let position = I16Vec3::new(reader.read_i16()?, reader.read_i16()?, reader.read_i16()?);

    let content_id = if serialization_version < 24 {
        u16::from(reader.read_u8()?)
    } else {
        reader.read_u16()?
    };

    let node = MapNode {
        content_id,
        param1: reader.read_u8()?,
        param2: reader.read_u8()?,
    };

    let keep_metadata = match reader.read_u8()? {
        0 => false,
        1 => true,
        _ => {
            return Err(ProtocolError::InvalidMapBlock("invalid keep_metadata flag"));
        }
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMapBlock(
            "trailing bytes in add-node packet",
        ));
    }

    Ok((position, node, keep_metadata))
}

pub fn decode_remove_node(data: &[u8]) -> Result<I16Vec3, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let position = I16Vec3::new(reader.read_i16()?, reader.read_i16()?, reader.read_i16()?);

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMapBlock(
            "trailing bytes in remove-node packet",
        ));
    }

    Ok(position)
}

pub fn encode_deleted_blocks(positions: &[I16Vec3]) -> Result<Bytes, ProtocolError> {
    let count = u8::try_from(positions.len()).map_err(|_| ProtocolError::InvalidBlockBatch)?;

    if count == 0 {
        return Err(ProtocolError::InvalidBlockBatch);
    }

    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::DeletedBlocks));
    writer.write_u8(count);

    for position in positions {
        writer.write_bytes(&position.x.to_be_bytes());
        writer.write_bytes(&position.y.to_be_bytes());
        writer.write_bytes(&position.z.to_be_bytes());
    }

    Ok(writer.into_inner())
}

pub fn encode_got_blocks(positions: &[I16Vec3]) -> Result<Bytes, ProtocolError> {
    let count = u8::try_from(positions.len()).map_err(|_| ProtocolError::InvalidBlockBatch)?;

    if count == 0 {
        return Err(ProtocolError::InvalidBlockBatch);
    }

    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::GotBlocks));
    writer.write_u8(count);

    for position in positions {
        writer.write_bytes(&position.x.to_be_bytes());
        writer.write_bytes(&position.y.to_be_bytes());
        writer.write_bytes(&position.z.to_be_bytes());
    }

    Ok(writer.into_inner())
}

fn decode_node_data(
    reader: &mut PacketReader<'_>,
    content_width: u8,
    params_width: u8,
) -> Result<Vec<MapNode>, ProtocolError> {
    if !matches!(content_width, 1 | 2) || params_width != 2 {
        return Err(ProtocolError::InvalidMapBlock(
            "unsupported node data width",
        ));
    }

    let mut content_ids = Vec::with_capacity(MAP_BLOCK_NODE_COUNT);

    for _ in 0..MAP_BLOCK_NODE_COUNT {
        let content_id = if content_width == 1 {
            u16::from(reader.read_u8()?)
        } else {
            reader.read_u16()?
        };

        content_ids.push(content_id);
    }

    let mut param1_values = Vec::with_capacity(MAP_BLOCK_NODE_COUNT);

    for _ in 0..MAP_BLOCK_NODE_COUNT {
        param1_values.push(reader.read_u8()?);
    }

    let mut param2_values = Vec::with_capacity(MAP_BLOCK_NODE_COUNT);

    for _ in 0..MAP_BLOCK_NODE_COUNT {
        param2_values.push(reader.read_u8()?);
    }

    let mut nodes = Vec::with_capacity(MAP_BLOCK_NODE_COUNT);

    for index in 0..MAP_BLOCK_NODE_COUNT {
        let mut content_id = content_ids[index];
        let mut param2 = param2_values[index];

        if content_width == 1 && content_id >= 0x80 {
            content_id = (content_id << 4) | u16::from(param2 >> 4);
            param2 &= 0x0F;
        }

        nodes.push(MapNode {
            content_id,
            param1: param1_values[index],
            param2,
        });
    }

    Ok(nodes)
}

fn decode_map_block_pre29(
    reader: &mut PacketReader<'_>,
    _serialization_version: u8,
) -> Result<(u8, u16, Vec<MapNode>, Vec<u8>), ProtocolError> {
    let flags = reader.read_u8()?;

    let lighting_complete = reader.read_u16()?;
    let content_width = reader.read_u8()?;
    let params_width = reader.read_u8()?;

    let compressed_parts = reader.read_bytes(reader.remaining())?;

    let (node_data, node_compressed_length) =
        decompress_zlib_frame(compressed_parts).map_err(ProtocolError::InvalidMapBlock)?;

    let remaining_data =
        compressed_parts
            .get(node_compressed_length..)
            .ok_or(ProtocolError::InvalidMapBlock(
                "invalid node compression length",
            ))?;

    let (metadata_data, metadata_compressed_length) =
        decompress_zlib_frame(remaining_data).map_err(ProtocolError::InvalidMapBlock)?;
    if node_compressed_length + metadata_compressed_length != compressed_parts.len() {
        return Err(ProtocolError::InvalidMapBlock(
            "unexpected bytes after compressed map block data",
        ));
    }

    let mut node_reader = PacketReader::new(&node_data);

    let nodes = decode_node_data(
        &mut node_reader,
        content_width,
        params_width,
    )?;

    if !node_reader.is_empty() {
        return Err(ProtocolError::InvalidMapBlock(
            "unexpected bytes after node data",
        ));
    }

    Ok((flags, lighting_complete, nodes, metadata_data))
}

pub fn decode_map_block(
    data: &[u8],
    serialization_version: u8,
) -> Result<MapBlockData, ProtocolError> {
    if !(27..=29).contains(&serialization_version) {
        return Err(ProtocolError::InvalidMapBlock(
            "unsupported serialization version",
        ));
    }

    let mut reader = PacketReader::new(data);

    if reader.remaining() < 9 {
        return Err(ProtocolError::InvalidMapBlock("packet too short"));
    }

    let position_bytes = reader.read_bytes(6)?;

    let position = I16Vec3::new(
        i16::from_be_bytes([position_bytes[0], position_bytes[1]]),
        i16::from_be_bytes([position_bytes[2], position_bytes[3]]),
        i16::from_be_bytes([position_bytes[4], position_bytes[5]]),
    );

    let body_length = reader.remaining() - 1;
    let serialized_body = reader.read_bytes(body_length)?;

    let network_specific_version = reader.read_u8()?;

    let (flags, lighting_complete, nodes, metadata_data) = if serialization_version == 29 {
        let decompressed =
            decompress_zstd_frame(serialized_body).map_err(ProtocolError::InvalidMapBlock)?;
        let mut block_reader = PacketReader::new(&decompressed);

        let flags = block_reader.read_u8()?;
        let lighting_complete = block_reader.read_u16()?;

        let content_width = block_reader.read_u8()?;
        let params_width = block_reader.read_u8()?;

        let nodes = decode_node_data(
            &mut block_reader,
            content_width,
            params_width,
        )?;

        let metadata_data = block_reader.read_bytes(block_reader.remaining())?.to_vec();

        (flags, lighting_complete, nodes, metadata_data)
    } else {
        let mut block_reader = PacketReader::new(serialized_body);

        decode_map_block_pre29(&mut block_reader, serialization_version)?
    };

    if flags & !0x0B != 0 {
        return Err(ProtocolError::InvalidMapBlock("invalid block flags"));
    }
    let metadata = decode_node_metadata_list(&metadata_data, false)?;

    Ok(MapBlockData {
        position,
        flags,
        lighting_complete,
        nodes,
        metadata_data,
        metadata,
        network_specific_version,
    })
}
