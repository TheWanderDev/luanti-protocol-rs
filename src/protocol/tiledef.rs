use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use bytes::Bytes;

const CONTENT_FEATURES_VERSION: u8 = 13;
const TILEDEF_VERSION: u8 = 6;
const NODE_TILE_COUNT: usize = 6;

const FLAG_BACKFACE_CULLING: u16 = 1 << 0;
const FLAG_TILEABLE_HORIZONTAL: u16 = 1 << 1;
const FLAG_TILEABLE_VERTICAL: u16 = 1 << 2;
const FLAG_HAS_COLOR: u16 = 1 << 3;
const FLAG_HAS_SCALE: u16 = 1 << 4;
const FLAG_HAS_ALIGN_STYLE: u16 = 1 << 5;
const NODE_OVERLAY_TILE_COUNT: usize = 6;
const NODE_SPECIAL_TILE_COUNT: usize = 6;
const NODE_PARAM_TYPE2_END: u8 = 14;
const NODE_DRAWTYPE_END: u8 = 18;
const NODE_LIQUID_TYPE_END: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeBoxPart {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub group: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeBoxDefinition {
    pub kind: u8,
    pub parts: Vec<NodeBoxPart>,
}

pub struct ContentFeaturesTiles {
    pub groups: Vec<(String, i16)>,
    pub param_type: u8,
    pub param_type2: u8,
    pub drawtype: u8,
    pub mesh_name: String,
    pub visual_scale: f32,
    pub alpha: u8,
    pub color: [u8; 3],
    pub palette_name: String,
    pub waving: u8,
    pub post_effect_color: [u8; 4],
    pub leveled: u8,
    pub connect_sides: u8,
    pub connect_ids: Vec<u16>,
    pub liquid_type: u8,
    pub liquid_range: u8,
    pub climbable: bool,
    pub liquid_viscosity: u8,
    pub liquid_renewable: bool,
    pub drowning: u8,
    pub liquid_floodable: bool,
    pub move_resistance: u8,
    pub liquid_move_physics: bool,
    pub liquid_alternative_flowing: String,
    pub liquid_alternative_source: String,
    pub node_box: NodeBoxDefinition,
    pub selection_box: NodeBoxDefinition,
    pub collision_box: NodeBoxDefinition,
    pub tiles: [TileDef; NODE_TILE_COUNT],
    pub overlays: [TileDef; NODE_OVERLAY_TILE_COUNT],
    pub special: [TileDef; NODE_SPECIAL_TILE_COUNT],
    pub light_propagates: bool,
    pub sunlight_propagates: bool,
    pub light_source: u8,
    pub is_ground_content: bool,
    pub walkable: bool,
    pub pointable: u8,
    pub diggable: bool,
    pub buildable_to: bool,
    pub rightclickable: bool,
    pub damage_per_second: u32,
    pub sounds: [SimpleSoundSpec; 3],
    pub legacy_facedir_simple: bool,
    pub legacy_wallmounted: bool,
    pub node_dig_prediction: String,
    pub leveled_max: u8,
    pub alpha_mode: u8,
    pub post_effect_color_shaded: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSoundSpec {
    pub name: String,
    pub parameters: [u8; 12],
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileAnimation {
    None,
    VerticalFrames {
        aspect_width: u16,
        aspect_height: u16,
        length: f32,
    },
    Sheet2d {
        frames_width: u8,
        frames_height: u8,
        frame_length: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileDef {
    pub name: String,
    pub animation: TileAnimation,
    pub backface_culling: bool,
    pub tileable_horizontal: bool,
    pub tileable_vertical: bool,
    pub color: Option<[u8; 3]>,
    pub scale: Option<u8>,
    pub align_style: Option<u8>,
}

fn read_node_box_vector(reader: &mut PacketReader<'_>) -> Result<[f32; 3], ProtocolError> {
    let value = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];

    if !value.iter().all(|component| component.is_finite()) {
        return Err(invalid_node_def("invalid node box coordinate"));
    }

    Ok(value.map(|component| component / 10.0))
}

fn read_node_box_part(
    reader: &mut PacketReader<'_>,
    group: u8,
) -> Result<NodeBoxPart, ProtocolError> {
    let min = read_node_box_vector(reader)?;
    let max = read_node_box_vector(reader)?;

    if !(0..3).all(|axis| min[axis] <= max[axis]) {
        return Err(invalid_node_def("invalid node box bounds"));
    }

    Ok(NodeBoxPart { min, max, group })
}

fn read_node_box(reader: &mut PacketReader<'_>) -> Result<NodeBoxDefinition, ProtocolError> {
    let version = reader.read_u8()?;

    if version < 6 {
        return Err(ProtocolError::UnsupportedNodeBoxVersion(version));
    }

    let raw_kind = reader.read_u8()?;
    let kind = if raw_kind <= 4 { raw_kind } else { 0 };
    let mut parts = Vec::new();

    match kind {
        0 => {}
        1 | 3 => {
            let count = usize::from(reader.read_u16()?);

            if count > reader.remaining() / 24 {
                return Err(invalid_node_def("node box count exceeds packet size"));
            }

            parts.reserve(count);
            for _ in 0..count {
                parts.push(read_node_box_part(reader, 0)?);
            }
        }
        2 => {
            for group in 0..3 {
                parts.push(read_node_box_part(reader, group)?);
            }
        }
        4 => {
            for group in 0..15_u8 {
                let count = usize::from(reader.read_u16()?);
                if count > reader.remaining() / 24 {
                    return Err(invalid_node_def(
                        "connected node box count exceeds packet size",
                    ));
                }

                for _ in 0..count {
                    parts.push(read_node_box_part(reader, group)?);
                }
            }
        }
        _ => return Err(ProtocolError::UnsupportedNodeBoxType(kind)),
    }

    Ok(NodeBoxDefinition { kind, parts })
}

struct NodeInteractionFields {
    is_ground_content: bool,
    walkable: bool,
    pointable: u8,
    diggable: bool,
    climbable: bool,
    buildable_to: bool,
    rightclickable: bool,
    damage_per_second: u32,
}

struct NodeMovementMetadata {
    legacy_facedir_simple: bool,
    legacy_wallmounted: bool,
    node_dig_prediction: String,
    leveled_max: u8,
    alpha_mode: u8,
    move_resistance: u8,
    liquid_move_physics: bool,
    post_effect_color_shaded: bool,
}

fn read_node_interaction_fields(
    reader: &mut PacketReader<'_>,
) -> Result<NodeInteractionFields, ProtocolError> {
    let is_ground_content = reader.read_u8()? != 0;
    let walkable = reader.read_u8()? != 0;
    let pointable = reader.read_u8()?;
    let diggable = reader.read_u8()? != 0;
    let climbable = reader.read_u8()? != 0;
    let buildable_to = reader.read_u8()? != 0;
    let rightclickable = reader.read_u8()? != 0;
    let damage_per_second = reader.read_u32()?;
    Ok(NodeInteractionFields {
        is_ground_content,
        walkable,
        pointable,
        diggable,
        climbable,
        buildable_to,
        rightclickable,
        damage_per_second,
    })
}

fn read_simple_sound_spec(reader: &mut PacketReader<'_>) -> Result<SimpleSoundSpec, ProtocolError> {
    let name = read_texture_name(reader)?;
    let mut parameters = [0_u8; 12];
    parameters.copy_from_slice(reader.read_bytes(12)?);
    Ok(SimpleSoundSpec { name, parameters })
}

const NODE_DRAWTYPE_NORMAL: u8 = 0;
const NODE_DRAWTYPE_LIQUID: u8 = 2;
const NODE_DRAWTYPE_FLOWING_LIQUID: u8 = 3;
const LEGACY_ALPHA_OPAQUE: u8 = u8::MAX;
const ALPHA_MODE_BLEND: u8 = 0;
const ALPHA_MODE_CLIP: u8 = 1;
const ALPHA_MODE_OPAQUE: u8 = 2;
const ALPHA_MODE_LEGACY_COMPAT: u8 = 3;
const ALPHA_MODE_END: u8 = 4;
const DEFAULT_LEVELED_MAX: u8 = 127;

fn legacy_alpha_mode(drawtype: u8, legacy_alpha: u8) -> u8 {
    let is_opaque = legacy_alpha == LEGACY_ALPHA_OPAQUE;

    match drawtype {
        NODE_DRAWTYPE_NORMAL => {
            if is_opaque {
                ALPHA_MODE_OPAQUE
            } else {
                ALPHA_MODE_CLIP
            }
        }
        NODE_DRAWTYPE_LIQUID | NODE_DRAWTYPE_FLOWING_LIQUID => {
            if is_opaque {
                ALPHA_MODE_OPAQUE
            } else {
                ALPHA_MODE_BLEND
            }
        }
        _ => {
            if is_opaque {
                ALPHA_MODE_CLIP
            } else {
                ALPHA_MODE_BLEND
            }
        }
    }
}

fn read_movement_metadata_tail(
    reader: &mut PacketReader<'_>,
    drawtype: u8,
    legacy_alpha: u8,
    liquid_type: u8,
    liquid_viscosity: u8,
) -> Result<NodeMovementMetadata, ProtocolError> {
    let legacy_facedir_simple = reader.read_u8()? != 0;
    let legacy_wallmounted = reader.read_u8()? != 0;
    let node_dig_prediction = read_texture_name(reader)?;

    let mut leveled_max = DEFAULT_LEVELED_MAX;
    let mut alpha_mode = legacy_alpha_mode(drawtype, legacy_alpha);
    let mut move_resistance = liquid_viscosity;
    let mut liquid_move_physics = matches!(liquid_type, 1 | 2);
    let mut post_effect_color_shaded = false;

    if reader.remaining() > 0 {
        leveled_max = reader.read_u8()?;
    }

    if reader.remaining() > 0 {
        let value = reader.read_u8()?;
        alpha_mode = if value >= ALPHA_MODE_END || value == ALPHA_MODE_LEGACY_COMPAT {
            ALPHA_MODE_OPAQUE
        } else {
            value
        };
    }

    if reader.remaining() > 0 {
        move_resistance = reader.read_u8()?;
        liquid_move_physics = reader.read_u8()? != 0;
    }

    if reader.remaining() > 0 {
        post_effect_color_shaded = reader.read_u8()? != 0;
    }

    Ok(NodeMovementMetadata {
        legacy_facedir_simple,
        legacy_wallmounted,
        node_dig_prediction,
        leveled_max,
        alpha_mode,
        move_resistance,
        liquid_move_physics,
        post_effect_color_shaded,
    })
}

fn invalid_node_def(reason: &'static str) -> ProtocolError {
    ProtocolError::InvalidNodeDef(reason)
}

fn read_texture_name(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| invalid_node_def("invalid tile texture name encoding"))
}

fn decode_animation(reader: &mut PacketReader<'_>) -> Result<TileAnimation, ProtocolError> {
    match reader.read_u8()? {
        0 => Ok(TileAnimation::None),

        1 => {
            let aspect_width = reader.read_u16()?;
            let aspect_height = reader.read_u16()?;
            let length = reader.read_f32()?;

            if aspect_width == 0 || aspect_height == 0 || !length.is_finite() || length <= 0.0 {
                return Err(invalid_node_def("invalid vertical tile animation"));
            }

            Ok(TileAnimation::VerticalFrames {
                aspect_width,
                aspect_height,
                length,
            })
        }

        2 => {
            let frames_width = reader.read_u8()?;
            let frames_height = reader.read_u8()?;
            let frame_length = reader.read_f32()?;

            if frames_width == 0
                || frames_height == 0
                || !frame_length.is_finite()
                || frame_length <= 0.0
            {
                return Err(invalid_node_def("invalid sheet tile animation"));
            }

            Ok(TileAnimation::Sheet2d {
                frames_width,
                frames_height,
                frame_length,
            })
        }

        _ => Ok(TileAnimation::None),
    }
}

fn decode_tile_def(reader: &mut PacketReader<'_>) -> Result<TileDef, ProtocolError> {
    let version = reader.read_u8()?;

    if version < TILEDEF_VERSION {
        return Err(invalid_node_def("unsupported TileDef version"));
    }

    let name = read_texture_name(reader)?;
    let animation = decode_animation(reader)?;
    let flags = reader.read_u16()?;

    let color = if flags & FLAG_HAS_COLOR != 0 {
        Some([reader.read_u8()?, reader.read_u8()?, reader.read_u8()?])
    } else {
        None
    };

    let scale = if flags & FLAG_HAS_SCALE != 0 {
        let value = reader.read_u8()?;

        if value == 0 {
            return Err(invalid_node_def("invalid TileDef scale"));
        }

        Some(value)
    } else {
        None
    };

    let align_style = if flags & FLAG_HAS_ALIGN_STYLE != 0 {
        let value = reader.read_u8()?;

        if value <= 2 && value != 0 {
            Some(value)
        } else {
            None
        }
    } else {
        None
    };

    Ok(TileDef {
        name,
        animation,
        backface_culling: flags & FLAG_BACKFACE_CULLING != 0,
        tileable_horizontal: flags & FLAG_TILEABLE_HORIZONTAL != 0,
        tileable_vertical: flags & FLAG_TILEABLE_VERTICAL != 0,
        color,
        scale,
        align_style,
    })
}

pub fn decode_content_features_tiles(
    features_version: u8,
    serialized_features: &[u8],
) -> Result<ContentFeaturesTiles, ProtocolError> {
    if features_version < CONTENT_FEATURES_VERSION {
        return Err(invalid_node_def(
            "unsupported content features version for tile decoding",
        ));
    }

    let mut reader = PacketReader::new(serialized_features);

    let group_count = usize::from(reader.read_u16()?);
    let mut groups = Vec::with_capacity(group_count);

    for _ in 0..group_count {
        let group_name = std::str::from_utf8(reader.read_string16()?)
            .map_err(|_| invalid_node_def("invalid node group name encoding"))?;
        let group_value = reader.read_i16()?;
        groups.push((group_name.to_owned(), group_value));
    }

    let param_type = reader.read_u8()?;
    let param_type = if param_type < 2 { param_type } else { 0 };

    let param_type2 = reader.read_u8()?;
    let param_type2 = if param_type2 < NODE_PARAM_TYPE2_END {
        param_type2
    } else {
        0
    };

    let drawtype = reader.read_u8()?;
    let drawtype = if drawtype < NODE_DRAWTYPE_END {
        drawtype
    } else {
        0
    };
    let mesh_name = read_texture_name(&mut reader)?;

    let visual_scale = reader.read_f32()?;

    if !visual_scale.is_finite() {
        return Err(invalid_node_def("invalid node visual scale"));
    }

    let tile_count = reader.read_u8()?;

    if usize::from(tile_count) != NODE_TILE_COUNT {
        return Err(invalid_node_def("unsupported node tile count"));
    }

    let mut tiles = Vec::with_capacity(NODE_TILE_COUNT);

    for _ in 0..NODE_TILE_COUNT {
        tiles.push(decode_tile_def(&mut reader)?);
    }

    let tiles = tiles
        .try_into()
        .map_err(|_| invalid_node_def("invalid decoded node tile count"))?;

    let mut overlays = Vec::with_capacity(NODE_OVERLAY_TILE_COUNT);

    for _ in 0..NODE_OVERLAY_TILE_COUNT {
        overlays.push(decode_tile_def(&mut reader)?);
    }

    let overlays = overlays
        .try_into()
        .map_err(|_| invalid_node_def("invalid decoded overlay tile count"))?;

    let special_count = usize::from(reader.read_u8()?);

    if special_count != NODE_SPECIAL_TILE_COUNT {
        return Err(invalid_node_def("unsupported special tile count"));
    }

    let mut special = Vec::with_capacity(NODE_SPECIAL_TILE_COUNT);

    for _ in 0..NODE_SPECIAL_TILE_COUNT {
        special.push(decode_tile_def(&mut reader)?);
    }

    let special = special
        .try_into()
        .map_err(|_| invalid_node_def("invalid decoded special tile count"))?;
    let alpha = reader.read_u8()?;
    let color = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];
    let palette_name = read_texture_name(&mut reader)?;
    let waving = reader.read_u8()?;
    let connect_sides = reader.read_u8()?;

    let connects_to_count = usize::from(reader.read_u16()?);

    if connects_to_count > reader.remaining() / 2 {
        return Err(invalid_node_def(
            "node connection count exceeds packet size",
        ));
    }

    let mut connect_ids = Vec::with_capacity(connects_to_count);
    for _ in 0..connects_to_count {
        connect_ids.push(reader.read_u16()?);
    }

    let mut post_effect_color = [0_u8; 4];
    post_effect_color.copy_from_slice(reader.read_bytes(4)?);
    let leveled = reader.read_u8()?;

    let light_propagates = reader.read_u8()? != 0;
    let sunlight_propagates = reader.read_u8()? != 0;
    let light_source = reader.read_u8()?.min(14);
    let interaction = read_node_interaction_fields(&mut reader)?;
    let liquid_type = reader.read_u8()?;
    let liquid_type = if liquid_type < NODE_LIQUID_TYPE_END {
        liquid_type
    } else {
        0
    };
    let liquid_alternative_flowing = read_texture_name(&mut reader)?;
    let liquid_alternative_source = read_texture_name(&mut reader)?;
    let liquid_viscosity = reader.read_u8()?;
    let liquid_renewable = reader.read_u8()? != 0;
    let liquid_range = reader.read_u8()?.clamp(1, 8);
    let drowning = reader.read_u8()?;
    let liquid_floodable = reader.read_u8()? != 0;
    let node_box = read_node_box(&mut reader)?;
    let selection_box = read_node_box(&mut reader)?;
    let collision_box = read_node_box(&mut reader)?;

    let sounds = [
        read_simple_sound_spec(&mut reader)?,
        read_simple_sound_spec(&mut reader)?,
        read_simple_sound_spec(&mut reader)?,
    ];
    let movement = read_movement_metadata_tail(
        &mut reader,
        drawtype,
        alpha,
        liquid_type,
        liquid_viscosity,
    )?;

    Ok(ContentFeaturesTiles {
        groups,
        param_type,
        param_type2,
        drawtype,
        mesh_name,
        visual_scale,
        alpha,
        color,
        palette_name,
        waving,
        post_effect_color,
        leveled,
        connect_sides,
        connect_ids,
        liquid_type,
        liquid_range,
        climbable: interaction.climbable,
        liquid_viscosity,
        liquid_renewable,
        drowning,
        liquid_floodable,
        move_resistance: movement.move_resistance,
        liquid_move_physics: movement.liquid_move_physics,
        liquid_alternative_flowing,
        liquid_alternative_source,
        node_box,
        selection_box,
        collision_box,
        tiles,
        overlays,
        special,
        light_propagates,
        sunlight_propagates,
        light_source,
        is_ground_content: interaction.is_ground_content,
        walkable: interaction.walkable,
        pointable: interaction.pointable,
        diggable: interaction.diggable,
        buildable_to: interaction.buildable_to,
        rightclickable: interaction.rightclickable,
        damage_per_second: interaction.damage_per_second,
        sounds,
        legacy_facedir_simple: movement.legacy_facedir_simple,
        legacy_wallmounted: movement.legacy_wallmounted,
        node_dig_prediction: movement.node_dig_prediction,
        leveled_max: movement.leveled_max,
        alpha_mode: movement.alpha_mode,
        post_effect_color_shaded: movement.post_effect_color_shaded,
    })
}

fn encode_node_box(
    writer: &mut PacketWriter,
    node_box: &NodeBoxDefinition,
) -> Result<(), ProtocolError> {
    if !node_box.parts.iter().all(|part| {
        part.min
            .into_iter()
            .chain(part.max)
            .all(|coordinate| coordinate.is_finite() && (coordinate * 10.0).is_finite())
            && (0..3).all(|axis| part.min[axis] <= part.max[axis])
            && (node_box.kind != 2 || part.group < 3)
            && (node_box.kind != 4 || part.group < 15)
    }) {
        return Err(invalid_node_def("invalid node box parts"));
    }

    match node_box.kind {
        0 if node_box.parts.is_empty() => {}
        1 | 3 if node_box.parts.len() <= usize::from(u16::MAX) => {}
        1 | 3 => return Err(invalid_node_def("too many node box parts")),
        2 if node_box.parts.len() == 3
            && (0..3).all(|group| {
                node_box
                    .parts
                    .iter()
                    .filter(|part| usize::from(part.group) == group)
                    .count()
                    > 0
            }) => {}
        4 if (0..15_u8).all(|group| {
            node_box
                .parts
                .iter()
                .filter(|part| part.group == group)
                .count()
                <= usize::from(u16::MAX)
        }) => {}
        4 => return Err(invalid_node_def("too many connected node box parts")),
        0 | 2 => return Err(invalid_node_def("invalid node box part count")),
        _ => return Err(ProtocolError::UnsupportedNodeBoxType(node_box.kind)),
    }

    writer.write_u8(TILEDEF_VERSION);
    writer.write_u8(node_box.kind);

    let write_part = |writer: &mut PacketWriter, part: &NodeBoxPart| {
        for coordinate in part.min.into_iter().chain(part.max) {
            writer.write_f32(coordinate * 10.0);
        }
    };

    match node_box.kind {
        0 => {}
        1 | 3 => {
            writer.write_u16(
                u16::try_from(node_box.parts.len())
                    .map_err(|_| invalid_node_def("too many node box parts"))?,
            );
            for part in &node_box.parts {
                write_part(writer, part);
            }
        }
        2 => {
            for group in 0..3 {
                if let Some(part) = node_box
                    .parts
                    .iter()
                    .find(|part| usize::from(part.group) == group)
                {
                    write_part(writer, part);
                }
            }
        }
        4 => {
            for group in 0..15_u8 {
                let count = node_box
                    .parts
                    .iter()
                    .filter(|part| part.group == group)
                    .count();
                writer.write_u16(
                    u16::try_from(count)
                        .map_err(|_| invalid_node_def("too many connected node box parts"))?,
                );
                for part in node_box.parts.iter().filter(|part| part.group == group) {
                    write_part(writer, part);
                }
            }
        }
        _ => return Err(ProtocolError::UnsupportedNodeBoxType(node_box.kind)),
    }

    Ok(())
}

fn encode_tile_def(
    writer: &mut PacketWriter,
    tile: &TileDef,
    protocol_version: u16,
) -> Result<(), ProtocolError> {
    writer.write_u8(TILEDEF_VERSION);
    let texture_name = if protocol_version > 39
        || tile.name.is_empty()
        || !tile.name.starts_with('[')
        || tile.name.starts_with("[combine:")
        || tile.name.starts_with("[inventorycube{")
        || tile.name.starts_with("[lowpart:")
    {
        tile.name.clone()
    } else {
        format!("blank.png^{}", tile.name)
    };
    writer.write_string16(&texture_name)?;

    match &tile.animation {
        TileAnimation::None => writer.write_u8(0),
        TileAnimation::VerticalFrames {
            aspect_width,
            aspect_height,
            length,
        } => {
            if *aspect_width == 0 || *aspect_height == 0 || !length.is_finite() || *length <= 0.0 {
                return Err(invalid_node_def("invalid vertical tile animation"));
            }
            writer.write_u8(1);
            writer.write_u16(*aspect_width);
            writer.write_u16(*aspect_height);
            writer.write_f32(*length);
        }
        TileAnimation::Sheet2d {
            frames_width,
            frames_height,
            frame_length,
        } => {
            if *frames_width == 0
                || *frames_height == 0
                || !frame_length.is_finite()
                || *frame_length <= 0.0
            {
                return Err(invalid_node_def("invalid sheet tile animation"));
            }
            writer.write_u8(2);
            writer.write_u8(*frames_width);
            writer.write_u8(*frames_height);
            writer.write_f32(*frame_length);
        }
    }

    let mut flags = 0_u16;
    if tile.backface_culling {
        flags |= FLAG_BACKFACE_CULLING;
    }
    if tile.tileable_horizontal {
        flags |= FLAG_TILEABLE_HORIZONTAL;
    }
    if tile.tileable_vertical {
        flags |= FLAG_TILEABLE_VERTICAL;
    }
    if tile.color.is_some() {
        flags |= FLAG_HAS_COLOR;
    }
    if tile.scale.is_some() {
        flags |= FLAG_HAS_SCALE;
    }
    if tile.align_style.is_some() {
        flags |= FLAG_HAS_ALIGN_STYLE;
    }
    writer.write_u16(flags);

    if let Some(color) = tile.color {
        for component in color {
            writer.write_u8(component);
        }
    }
    if let Some(scale) = tile.scale {
        if scale == 0 {
            return Err(invalid_node_def("invalid TileDef scale"));
        }
        writer.write_u8(scale);
    }
    if let Some(align_style) = tile.align_style {
        if align_style > 2 {
            return Err(invalid_node_def("invalid TileDef alignment style"));
        }
        writer.write_u8(align_style);
    }

    Ok(())
}

pub fn encode_content_features_tiles(
    features: &ContentFeaturesTiles,
    protocol_version: u16,
) -> Result<Bytes, ProtocolError> {
    if features.groups.len() > usize::from(u16::MAX)
        || features.connect_ids.len() > usize::from(u16::MAX)
        || !features.visual_scale.is_finite()
        || features.light_source > 14
        || features.pointable > 2
        || features.param_type > 1
        || features.liquid_range == 0
        || features.liquid_range > 8
        || !features.groups.iter().all(|(name, _)| !name.is_empty())
    {
        return Err(invalid_node_def("invalid content feature values"));
    }

    let mut writer = PacketWriter::with_capacity(1024);
    writer.write_u16(
        u16::try_from(features.groups.len())
            .map_err(|_| invalid_node_def("too many node groups"))?,
    );
    for (name, value) in &features.groups {
        writer.write_string16(name)?;
        let value = if protocol_version < 41 && name == "bouncy" {
            value.wrapping_abs()
        } else {
            *value
        };
        writer.write_i16(value);
    }

    writer.write_u8(features.param_type);
    writer.write_u8(features.param_type2);
    writer.write_u8(features.drawtype);
    writer.write_string16(&features.mesh_name)?;
    writer.write_f32(features.visual_scale);
    writer.write_u8(
        u8::try_from(NODE_TILE_COUNT).map_err(|_| invalid_node_def("invalid node tile count"))?,
    );
    for tile in &features.tiles {
        encode_tile_def(&mut writer, tile, protocol_version)?;
    }
    for tile in &features.overlays {
        encode_tile_def(&mut writer, tile, protocol_version)?;
    }
    writer.write_u8(
        u8::try_from(NODE_SPECIAL_TILE_COUNT)
            .map_err(|_| invalid_node_def("invalid special tile count"))?,
    );
    for tile in &features.special {
        encode_tile_def(&mut writer, tile, protocol_version)?;
    }

    writer.write_u8(features.alpha);
    for component in features.color {
        writer.write_u8(component);
    }
    writer.write_string16(&features.palette_name)?;
    writer.write_u8(features.waving);
    writer.write_u8(features.connect_sides);
    writer.write_u16(
        u16::try_from(features.connect_ids.len())
            .map_err(|_| invalid_node_def("too many node connection identifiers"))?,
    );
    for content_id in &features.connect_ids {
        writer.write_u16(*content_id);
    }

    for component in features.post_effect_color {
        writer.write_u8(component);
    }
    writer.write_u8(features.leveled);
    writer.write_u8(u8::from(features.light_propagates));
    writer.write_u8(u8::from(features.sunlight_propagates));
    writer.write_u8(features.light_source);

    writer.write_u8(u8::from(features.is_ground_content));
    writer.write_u8(u8::from(features.walkable));
    writer.write_u8(features.pointable);
    writer.write_u8(u8::from(features.diggable));
    writer.write_u8(u8::from(features.climbable));
    writer.write_u8(u8::from(features.buildable_to));
    writer.write_u8(u8::from(features.rightclickable));
    writer.write_u32(features.damage_per_second);

    let liquid_type = if protocol_version <= 39 {
        match features.drawtype {
            2 => 1,
            3 => 2,
            _ => features.liquid_type,
        }
    } else {
        features.liquid_type
    };
    writer.write_u8(liquid_type);
    writer.write_string16(&features.liquid_alternative_flowing)?;
    writer.write_string16(&features.liquid_alternative_source)?;
    writer.write_u8(features.liquid_viscosity);
    writer.write_u8(u8::from(features.liquid_renewable));
    writer.write_u8(features.liquid_range);
    writer.write_u8(features.drowning);
    writer.write_u8(u8::from(features.liquid_floodable));

    encode_node_box(&mut writer, &features.node_box)?;
    encode_node_box(&mut writer, &features.selection_box)?;
    encode_node_box(&mut writer, &features.collision_box)?;

    for sound in &features.sounds {
        writer.write_string16(&sound.name)?;
        writer.write_bytes(&sound.parameters);
    }

    writer.write_u8(u8::from(features.legacy_facedir_simple));
    writer.write_u8(u8::from(features.legacy_wallmounted));
    writer.write_string16(&features.node_dig_prediction)?;
    writer.write_u8(features.leveled_max);
    writer.write_u8(features.alpha_mode);
    writer.write_u8(features.move_resistance);
    writer.write_u8(u8::from(features.liquid_move_physics));
    writer.write_u8(u8::from(features.post_effect_color_shaded));

    Ok(writer.into_inner())
}
