use crate::errors::ProtocolError;
use crate::protocol::compression::decompress_limited;
use crate::protocol::node_transform::NodeTransform;
use crate::protocol::reader::PacketReader;
use crate::protocol::tiledef::{
    NodeBoxDefinition, NodeBoxPart, decode_content_features_tiles, encode_content_features_tiles,
};
use crate::protocol::writer::PacketWriter;
use ahash::{AHashMap, AHashSet};
use bytes::Bytes;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::{Cursor, Write};

const NODEDEF_MANAGER_VERSION: u8 = 1;
const CONTENT_FEATURES_VERSION: u8 = 13;
const MAX_NODEDEF_SIZE: usize = 32 * 1024 * 1024;
const CONTENT_UNKNOWN: u16 = 125;
const CONTENT_AIR: u16 = 126;
pub const CONTENT_IGNORE: u16 = 127;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeLightingFlags {
    pub has_light: bool,
    pub light_propagates: bool,
    pub sunlight_propagates: bool,
    pub light_source: u8,
    pub walkable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeDefinition {
    pub content_id: u16,
    pub name: String,
    pub features_version: u8,
    pub serialized_features: Vec<u8>,
    pub node_box: NodeBoxDefinition,
    pub collision_box: NodeBoxDefinition,
    pub selection_box: NodeBoxDefinition,
    pub param_type2: u8,
    pub leveled: u8,
    pub connect_ids: Vec<u16>,
    pub climbable: bool,
    pub liquid_type: u8,
    pub liquid_viscosity: u8,
    pub liquid_floodable: bool,
    pub move_resistance: u8,
    pub liquid_move_physics: bool,
}

fn skip_tile_definition(reader: &mut PacketReader<'_>) -> Option<()> {
    reader.read_u8().ok()?;
    reader.read_string16().ok()?;

    match reader.read_u8().ok()? {
        0 => {}
        1 => {
            reader.read_bytes(8).ok()?;
        }
        2 => {
            reader.read_bytes(6).ok()?;
        }
        _ => return None,
    }

    let flags = reader.read_u16().ok()?;

    if flags & 0x08 != 0 {
        reader.read_bytes(3).ok()?;
    }

    if flags & 0x10 != 0 {
        reader.read_u8().ok()?;
    }

    if flags & 0x20 != 0 {
        reader.read_u8().ok()?;
    }

    Some(())
}

fn parse_lighting_flags(data: &[u8]) -> Option<NodeLightingFlags> {
    let mut reader = PacketReader::new(data);
    let group_count = usize::from(reader.read_u16().ok()?);

    for _ in 0..group_count {
        reader.read_string16().ok()?;
        reader.read_i16().ok()?;
    }

    let param_type = reader.read_u8().ok()?;
    reader.read_u8().ok()?;
    reader.read_u8().ok()?;
    reader.read_string16().ok()?;
    reader.read_f32().ok()?;

    if reader.read_u8().ok()? != 6 {
        return None;
    }

    for _ in 0..12 {
        skip_tile_definition(&mut reader)?;
    }

    let special_count = reader.read_u8().ok()?;
    if special_count != 6 {
        return None;
    }

    for _ in 0..usize::from(special_count) {
        skip_tile_definition(&mut reader)?;
    }

    reader.read_bytes(4).ok()?;
    reader.read_string16().ok()?;
    reader.read_bytes(2).ok()?;

    let connect_count = usize::from(reader.read_u16().ok()?);
    let connect_bytes = connect_count.checked_mul(2)?;
    reader.read_bytes(connect_bytes).ok()?;

    reader.read_bytes(4).ok()?;
    reader.read_u8().ok()?;

    let light_propagates = reader.read_u8().ok()? != 0;
    let sunlight_propagates = reader.read_u8().ok()? != 0;
    let light_source = reader.read_u8().ok()?.min(14);
    reader.read_u8().ok()?;
    let walkable = reader.read_u8().ok()? != 0;

    Some(NodeLightingFlags {
        has_light: param_type == 1,
        light_propagates,
        sunlight_propagates,
        light_source,
        walkable,
    })
}

fn annotate_node_feature_error(content_id: u16, name: &str, error: ProtocolError) -> ProtocolError {
    let reason = match error {
        ProtocolError::InvalidNodeDef(reason) => reason.to_owned(),
        ProtocolError::UnsupportedNodeBoxType(kind) => format!("unsupported node box type {kind}"),
        ProtocolError::UnsupportedNodeBoxVersion(version) => {
            format!("unsupported node box version {version}")
        }
        other => return other,
    };

    ProtocolError::InvalidNodeDefContext {
        content_id,
        name: name.to_owned(),
        reason,
    }
}

const PARAM_TYPE2_WALLMOUNTED: u8 = 4;
const PARAM_TYPE2_COLORED_WALLMOUNTED: u8 = 10;

fn wallmounted_direction(param_type2: u8, param2: u8) -> u8 {
    if matches!(
        param_type2,
        PARAM_TYPE2_WALLMOUNTED | PARAM_TYPE2_COLORED_WALLMOUNTED
    ) {
        param2 & 0x07
    } else {
        0
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeRegistry {
    by_id: AHashMap<u16, NodeDefinition>,
    by_name: AHashMap<String, u16>,
    walkable_by_id: AHashMap<u16, bool>,
}

impl NodeRegistry {
    pub fn from_definitions(definitions: Vec<NodeDefinition>) -> Result<Self, ProtocolError> {
        let mut registry = Self::default();
        for (content_id, name) in [
            (CONTENT_UNKNOWN, "unknown"),
            (CONTENT_AIR, "air"),
            (CONTENT_IGNORE, "ignore"),
        ] {
            let definition = NodeDefinition {
                content_id,
                name: name.to_owned(),
                features_version: CONTENT_FEATURES_VERSION,
                serialized_features: Vec::new(),
                node_box: NodeBoxDefinition {
                    kind: 0,
                    parts: Vec::new(),
                },
                collision_box: NodeBoxDefinition {
                    kind: 0,
                    parts: Vec::new(),
                },
                selection_box: NodeBoxDefinition {
                    kind: 0,
                    parts: Vec::new(),
                },
                param_type2: 0,
                leveled: 0,
                connect_ids: Vec::new(),
                climbable: false,
                liquid_type: 0,
                liquid_viscosity: 0,
                liquid_floodable: false,
                move_resistance: 0,
                liquid_move_physics: false,
            };

            registry.by_name.insert(name.to_owned(), content_id);
            registry
                .walkable_by_id
                .insert(content_id, name != "air" && name != "ignore");
            registry.by_id.insert(content_id, definition);
        }

        for definition in definitions {
            let content_id = definition.content_id;
            let name = definition.name.clone();

            if registry.by_id.contains_key(&content_id) || registry.by_name.contains_key(&name) {
                return Err(ProtocolError::InvalidNodeDef("duplicate node definition"));
            }
            let Some(flags) = parse_lighting_flags(&definition.serialized_features) else {
                return Err(ProtocolError::InvalidNodeDef(
                    "invalid node lighting features",
                ));
            };

            registry.walkable_by_id.insert(content_id, flags.walkable);
            registry.by_name.insert(name, content_id);
            registry.by_id.insert(content_id, definition);
        }

        Ok(registry)
    }

    pub fn collision_boxes(
        &self,
        content_id: u16,
        param2: u8,
        neighbors: [Option<u16>; 6],
    ) -> Option<Vec<NodeBoxPart>> {
        let definition = self.by_id.get(&content_id)?;
        if definition.name == "ignore" {
            return None;
        }
        if !self.is_walkable(content_id) {
            return Some(Vec::new());
        }

        let shape =
            if definition.collision_box.kind == 1 && definition.collision_box.parts.is_empty() {
                &definition.node_box
            } else {
                &definition.collision_box
            };
        let mut parts = match shape.kind {
            0 => vec![NodeBoxPart {
                min: [-0.5; 3],
                max: [0.5; 3],
                group: 0,
            }],
            1 => shape
                .parts
                .iter()
                .filter(|part| part.group == 0)
                .copied()
                .collect(),
            2 => {
                let group = match wallmounted_direction(definition.param_type2, param2) {
                    0 | 6 => 0,
                    1 | 7 => 1,
                    _ => 2,
                };
                shape
                    .parts
                    .iter()
                    .filter(|part| part.group == group)
                    .copied()
                    .collect()
            }
            3 => {
                let mut fixed: Vec<NodeBoxPart> = shape
                    .parts
                    .iter()
                    .filter(|part| part.group == 0)
                    .copied()
                    .collect();
                let level = if definition.leveled > 0 {
                    definition.leveled
                } else {
                    param2.min(127)
                };
                let top = -0.5 + f32::from(level) / 64.0;
                for part in &mut fixed {
                    part.max[1] = part.max[1].min(top);
                }
                fixed.retain(|part| part.min[1] < part.max[1]);
                fixed
            }
            4 => {
                let connected: [bool; 6] = std::array::from_fn(|index| {
                    neighbors[index]
                        .is_some_and(|neighbor| definition.connect_ids.contains(&neighbor))
                });
                let has_connection = connected.contains(&true);
                let has_side_connection = connected[2..].contains(&true);
                shape
                    .parts
                    .iter()
                    .filter(|part| match part.group {
                        0 => true,
                        1..=6 => connected[usize::from(part.group - 1)],
                        7..=12 => !connected[usize::from(part.group - 7)],
                        13 => !has_connection,
                        14 => !has_side_connection,
                        _ => false,
                    })
                    .copied()
                    .collect()
            }
            _ => return None,
        };

        if shape.kind == 2 {
            let wall = wallmounted_direction(definition.param_type2, param2);
            let (group, turns) = match wall {
                0 => (0, 0),
                1 => (1, 0),
                2 => (2, 2),
                3 => (2, 0),
                4 => (2, 1),
                5 => (2, 3),
                6 => (0, 3),
                7 => (1, 1),
                _ => (0, 0),
            };
            let transform = NodeTransform::from_y_quarters(turns);
            for part in parts.iter_mut().filter(|part| part.group == group) {
                if turns != 0 {
                    (part.min, part.max) = transform.transform_bounds(part.min, part.max);
                }
            }
        } else if shape.kind != 4 {
            let transform = NodeTransform::from_param_type2(definition.param_type2, param2);
            for part in &mut parts {
                (part.min, part.max) = transform.transform_bounds(part.min, part.max);
            }
        }

        Some(parts)
    }

    #[must_use]
    pub fn selection_boxes(
        &self,
        content_id: u16,
        param2: u8,
        neighbors: [Option<u16>; 6],
    ) -> Option<Vec<NodeBoxPart>> {
        let definition = self.by_id.get(&content_id)?;
        if definition.name == "ignore" {
            return None;
        }
        if definition.name == "air" {
            return Some(Vec::new());
        }

        let shape = &definition.selection_box;
        let mut parts = match shape.kind {
            0 => vec![NodeBoxPart {
                min: [-0.5; 3],
                max: [0.5; 3],
                group: 0,
            }],
            1 => shape
                .parts
                .iter()
                .filter(|part| part.group == 0)
                .copied()
                .collect(),
            2 => {
                let group = match wallmounted_direction(definition.param_type2, param2) {
                    0 | 6 => 0,
                    1 | 7 => 1,
                    _ => 2,
                };
                shape
                    .parts
                    .iter()
                    .filter(|part| part.group == group)
                    .copied()
                    .collect()
            }
            3 => {
                let mut fixed: Vec<NodeBoxPart> = shape
                    .parts
                    .iter()
                    .filter(|part| part.group == 0)
                    .copied()
                    .collect();
                let level = if definition.leveled > 0 {
                    definition.leveled
                } else {
                    param2.min(127)
                };
                let top = -0.5 + f32::from(level) / 64.0;
                for part in &mut fixed {
                    part.max[1] = part.max[1].min(top);
                }
                fixed.retain(|part| part.min[1] < part.max[1]);
                fixed
            }
            4 => {
                let connected: [bool; 6] = std::array::from_fn(|index| {
                    neighbors[index]
                        .is_some_and(|neighbor| definition.connect_ids.contains(&neighbor))
                });
                let has_connection = connected.contains(&true);
                let has_side_connection = connected[2..].contains(&true);
                shape
                    .parts
                    .iter()
                    .filter(|part| match part.group {
                        0 => true,
                        1..=6 => connected[usize::from(part.group - 1)],
                        7..=12 => !connected[usize::from(part.group - 7)],
                        13 => !has_connection,
                        14 => !has_side_connection,
                        _ => false,
                    })
                    .copied()
                    .collect()
            }
            _ => return None,
        };

        if shape.kind == 2 {
            let wall = wallmounted_direction(definition.param_type2, param2);
            let (group, turns) = match wall {
                0 => (0, 0),
                1 => (1, 0),
                2 => (2, 2),
                3 => (2, 0),
                4 => (2, 1),
                5 => (2, 3),
                6 => (0, 3),
                7 => (1, 1),
                _ => (0, 0),
            };
            let transform = NodeTransform::from_y_quarters(turns);
            for part in parts.iter_mut().filter(|part| part.group == group) {
                if turns != 0 {
                    (part.min, part.max) = transform.transform_bounds(part.min, part.max);
                }
            }
        } else if shape.kind != 4 {
            let transform = NodeTransform::from_param_type2(definition.param_type2, param2);
            for part in &mut parts {
                (part.min, part.max) = transform.transform_bounds(part.min, part.max);
            }
        }

        Some(parts)
    }

    pub fn iter(&self) -> impl Iterator<Item = &NodeDefinition> {
        self.by_id.values()
    }

    #[must_use]
    pub fn get_by_id(&self, content_id: u16) -> Option<&NodeDefinition> {
        self.by_id.get(&content_id)
    }

    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&NodeDefinition> {
        self.by_name
            .get(name)
            .and_then(|content_id| self.by_id.get(content_id))
    }

    #[must_use]
    pub fn lighting_flags(&self, content_id: u16) -> NodeLightingFlags {
        let Some(definition) = self.by_id.get(&content_id) else {
            return NodeLightingFlags::default();
        };

        if definition.name == "unknown" && definition.serialized_features.is_empty() {
            return NodeLightingFlags {
                has_light: false,
                light_propagates: true,
                sunlight_propagates: true,
                light_source: 0,
                walkable: true,
            };
        }

        if definition.name == "air" && definition.serialized_features.is_empty() {
            return NodeLightingFlags {
                has_light: true,
                light_propagates: true,
                sunlight_propagates: true,
                light_source: 0,
                walkable: false,
            };
        }

        parse_lighting_flags(&definition.serialized_features).unwrap_or_default()
    }

    #[must_use]
    pub fn is_walkable(&self, content_id: u16) -> bool {
        self.walkable_by_id
            .get(&content_id)
            .copied()
            .unwrap_or(true)
    }

    #[must_use]
    pub fn movement_metadata(&self, content_id: u16) -> Option<&NodeDefinition> {
        self.by_id.get(&content_id)
    }

    #[must_use]
    pub fn movement_metadata_by_name(&self, name: &str) -> Option<&NodeDefinition> {
        self.by_name
            .get(name)
            .and_then(|content_id| self.by_id.get(content_id))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

pub fn encode_node_def(
    definitions: &[NodeDefinition],
    protocol_version: u16,
) -> Result<Bytes, ProtocolError> {
    let count = u16::try_from(definitions.len())
        .map_err(|_| ProtocolError::InvalidNodeDef("too many node definitions"))?;
    let mut body = PacketWriter::with_capacity(definitions.len().saturating_mul(128));
    let mut ids = AHashSet::with_capacity(definitions.len());
    let mut names = AHashSet::with_capacity(definitions.len());
    let mut body_size = 0_usize;

    for definition in definitions {
        if !ids.insert(definition.content_id) || !names.insert(definition.name.as_str()) {
            return Err(ProtocolError::InvalidNodeDef(
                "duplicate node definition identifier or name",
            ));
        }
        if definition.name.is_empty() || definition.features_version < CONTENT_FEATURES_VERSION {
            return Err(ProtocolError::InvalidNodeDef(
                "invalid node definition name or features version",
            ));
        }

        let decoded_features = decode_content_features_tiles(
            definition.features_version,
            &definition.serialized_features,
        )?;
        let serialized_features =
            encode_content_features_tiles(&decoded_features, protocol_version)?;

        let mut features = PacketWriter::with_capacity(
            serialized_features
                .len()
                .saturating_add(definition.name.len())
                .saturating_add(3),
        );
        features.write_u8(definition.features_version);
        features.write_string16(&definition.name)?;
        features.write_bytes(&serialized_features);
        let features = features.into_inner();
        let features_length = u16::try_from(features.len())
            .map_err(|_| ProtocolError::InvalidNodeDef("node features exceed string16 limit"))?;
        body_size = body_size
            .checked_add(features.len().saturating_add(4))
            .filter(|size| *size <= MAX_NODEDEF_SIZE - 7)
            .ok_or(ProtocolError::InvalidNodeDef(
                "node definition data exceeds size limit",
            ))?;

        body.write_u16(definition.content_id);
        body.write_u16(features_length);
        body.write_bytes(&features);
    }

    let body = body.into_inner();
    let body_length = u32::try_from(body.len())
        .map_err(|_| ProtocolError::InvalidNodeDef("node definition body is too large"))?;
    let mut manager = PacketWriter::with_capacity(body.len().saturating_add(7));
    manager.write_u8(NODEDEF_MANAGER_VERSION);
    manager.write_u16(count);
    manager.write_u32(body_length);
    manager.write_bytes(&body);
    let manager = manager.into_inner();

    if manager.len() > MAX_NODEDEF_SIZE {
        return Err(ProtocolError::InvalidNodeDef(
            "node definition data exceeds size limit",
        ));
    }

    let compressed = if protocol_version >= crate::protocol::compression::ZSTD_PROTOCOL_VERSION {
        zstd::encode_all(Cursor::new(manager), 3)
            .map_err(|_| ProtocolError::InvalidNodeDef("node definition compression failed"))?
    } else {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&manager)
            .map_err(|_| ProtocolError::InvalidNodeDef("node definition compression failed"))?;
        encoder
            .finish()
            .map_err(|_| ProtocolError::InvalidNodeDef("node definition compression failed"))?
    };

    if compressed.len() > MAX_NODEDEF_SIZE {
        return Err(ProtocolError::InvalidNodeDef(
            "compressed node definition data exceeds size limit",
        ));
    }

    let compressed_length = u32::try_from(compressed.len())
        .map_err(|_| ProtocolError::InvalidNodeDef("compressed node definition is too large"))?;
    let mut packet = PacketWriter::with_capacity(compressed.len().saturating_add(4));
    packet.write_u32(compressed_length);
    packet.write_bytes(&compressed);
    Ok(packet.into_inner())
}

pub(crate) fn decode_node_def(
    data: &[u8],
    protocol_version: u16,
) -> Result<Vec<NodeDefinition>, ProtocolError> {
    let mut packet_reader = PacketReader::new(data);

    let compressed_length = usize::try_from(packet_reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidNodeDef("invalid blob length"))?;

    if compressed_length > MAX_NODEDEF_SIZE {
        return Err(ProtocolError::InvalidNodeDef(
            "compressed blob exceeds size limit",
        ));
    }

    let compressed_data = packet_reader.read_bytes(compressed_length)?;

    if !packet_reader.is_empty() {
        return Err(ProtocolError::InvalidNodeDef(
            "unexpected bytes after compressed blob",
        ));
    }

    let decompressed = decompress_limited(compressed_data, MAX_NODEDEF_SIZE, protocol_version)
        .map_err(ProtocolError::InvalidNodeDef)?;

    let mut reader = PacketReader::new(&decompressed);

    let manager_version = reader.read_u8()?;

    if manager_version < NODEDEF_MANAGER_VERSION {
        return Err(ProtocolError::InvalidNodeDef(
            "unsupported node definition manager version",
        ));
    }

    let count = usize::from(reader.read_u16()?);

    let body_length = usize::try_from(reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidNodeDef("invalid body length"))?;

    let body = reader.read_bytes(body_length)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidNodeDef(
            "unexpected bytes after node definition body",
        ));
    }

    if count > body.len() / 4 {
        return Err(ProtocolError::InvalidNodeDef(
            "node definition count exceeds body size",
        ));
    }

    let mut body_reader = PacketReader::new(body);
    let mut definitions = Vec::with_capacity(count);

    for _ in 0..count {
        let content_id = body_reader.read_u16()?;
        let features_data = body_reader.read_string16()?;

        let mut features_reader = PacketReader::new(features_data);

        let features_version = features_reader.read_u8()?;

        if features_version < CONTENT_FEATURES_VERSION {
            return Err(ProtocolError::InvalidNodeDef(
                "unsupported content features version",
            ));
        }

        let name_bytes = features_reader.read_string16()?;

        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| ProtocolError::InvalidNodeDef("invalid node name encoding"))?
            .to_owned();

        if name.is_empty() {
            return Err(ProtocolError::InvalidNodeDef("empty node name"));
        }

        let serialized_features = features_reader
            .read_bytes(features_reader.remaining())?
            .to_vec();
        let decoded_features =
            decode_content_features_tiles(features_version, &serialized_features)
                .map_err(|error| annotate_node_feature_error(content_id, &name, error))?;

        definitions.push(NodeDefinition {
            content_id,
            name,
            features_version,
            serialized_features,
            node_box: decoded_features.node_box,
            collision_box: decoded_features.collision_box,
            selection_box: decoded_features.selection_box,
            param_type2: decoded_features.param_type2,
            leveled: decoded_features.leveled,
            connect_ids: decoded_features.connect_ids,
            climbable: decoded_features.climbable,
            liquid_type: decoded_features.liquid_type,
            liquid_viscosity: decoded_features.liquid_viscosity,
            liquid_floodable: decoded_features.liquid_floodable,
            move_resistance: decoded_features.move_resistance,
            liquid_move_physics: decoded_features.liquid_move_physics,
        });
    }

    if !body_reader.is_empty() {
        return Err(ProtocolError::InvalidNodeDef(
            "unexpected bytes after node definitions",
        ));
    }

    Ok(definitions)
}
