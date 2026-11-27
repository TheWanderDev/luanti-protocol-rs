use std::io::{Cursor, Write};

use crate::errors::ProtocolError;
use crate::protocol::compression::decompress_limited;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use ahash::AHashMap;
use bytes::Bytes;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use glam::Vec3;
const ITEMDEF_MANAGER_VERSION: u8 = 0;
const ITEM_DEFINITION_VERSION: u8 = 6;
const MAX_ITEMDEF_SIZE: usize = 32 * 1024 * 1024;

pub const ITEM_TYPE_TOOL: u8 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemDefinition {
    pub item_type: u8,
    pub name: String,
    pub description: String,
    pub short_description: String,

    pub inventory_image: ItemImageDef,
    pub inventory_overlay: ItemImageDef,
    pub wield_image: ItemImageDef,
    pub wield_overlay: ItemImageDef,

    pub wield_scale: Vec3,
    pub stack_max: u16,
    pub usable: bool,
    pub liquids_pointable: bool,

    pub tool_capabilities: Option<ToolCapabilities>,
    pub groups: Vec<(String, i16)>,

    pub node_placement_prediction: String,
    pub sound_place: SimpleSoundSpec,
    pub sound_place_failed: SimpleSoundSpec,
    pub sound_use: SimpleSoundSpec,
    pub sound_use_air: SimpleSoundSpec,

    pub range: f32,
    pub palette_image: String,
    pub color: [u8; 4],

    pub place_param2: Option<u8>,
    pub wallmounted_rotate_vertical: bool,
    pub touch_interaction: [u8; 3],

    pub pointabilities: Option<Pointabilities>,
    pub wear_bar_params: Option<WearBarParams>,
}
#[derive(Debug, Clone, PartialEq)]
struct ItemDefinitionExtensions {
    short_description: String,
    place_param2: Option<u8>,
    sound_use: SimpleSoundSpec,
    sound_use_air: SimpleSoundSpec,
    wallmounted_rotate_vertical: bool,
    touch_interaction: [u8; 3],
    pointabilities: Option<Pointabilities>,
    wear_bar_params: Option<WearBarParams>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemImageDef {
    pub name: String,
    pub animation: TileAnimation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileAnimation {
    None,
    VerticalFrames {
        aspect_w: u16,
        aspect_h: u16,
        length: f32,
    },
    Sheet2d {
        frames_w: u8,
        frames_h: u8,
        frame_length: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCapabilities {
    pub full_punch_interval: f32,
    pub max_drop_level: i16,
    pub groupcaps: Vec<ToolGroupCapability>,
    pub damage_groups: Vec<(String, i16)>,
    pub punch_attack_uses: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolGroupCapability {
    pub name: String,
    pub uses: i16,
    pub max_level: i16,
    pub times: Vec<(i16, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleSoundSpec {
    pub name: String,
    pub gain: f32,
    pub pitch: f32,
    pub fade: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum PointabilityType {
    NotPointable = 0,
    Pointable = 1,
    Blocking = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pointabilities {
    pub nodes: Vec<(String, PointabilityType)>,
    pub node_groups: Vec<(String, PointabilityType)>,
    pub objects: Vec<(String, PointabilityType)>,
    pub object_groups: Vec<(String, PointabilityType)>,
}

impl Pointabilities {
    #[must_use]
    pub fn node_rule(&self, name: &str) -> Option<PointabilityType> {
        self.nodes
            .iter()
            .find(|(rule_name, _)| rule_name == name)
            .map(|(_, value)| *value)
    }

    #[must_use]
    pub fn node_group_rule(&self, group: &str) -> Option<PointabilityType> {
        self.node_groups
            .iter()
            .find(|(rule_name, _)| rule_name == group)
            .map(|(_, value)| *value)
    }

    #[must_use]
    pub fn object_rule(&self, name: &str) -> Option<PointabilityType> {
        self.objects
            .iter()
            .find(|(rule_name, _)| rule_name == name)
            .map(|(_, value)| *value)
    }

    #[must_use]
    pub fn object_group_rule(&self, group: &str) -> Option<PointabilityType> {
        self.object_groups
            .iter()
            .find(|(rule_name, _)| rule_name == group)
            .map(|(_, value)| *value)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
            && self.node_groups.is_empty()
            && self.objects.is_empty()
            && self.object_groups.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum WearBarBlendMode {
    Constant = 0,
    Linear = 1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WearBarParams {
    pub color_stops: Vec<(f32, [u8; 4])>,
    pub blend: WearBarBlendMode,
}

impl WearBarParams {
    #[must_use]
    pub fn color_at(&self, durability: f32) -> Option<[u8; 4]> {
        if self.color_stops.is_empty() || !durability.is_finite() {
            return None;
        }

        let position = durability.clamp(0.0, 1.0);

        if self.blend == WearBarBlendMode::Constant {
            return self
                .color_stops
                .iter()
                .rev()
                .find(|(stop, _)| *stop <= position)
                .or_else(|| self.color_stops.first())
                .map(|(_, color)| *color);
        }

        let first = self.color_stops.first()?;

        if position <= first.0 {
            return Some(first.1);
        }

        let last = self.color_stops.last()?;

        if position >= last.0 {
            return Some(last.1);
        }

        for pair in self.color_stops.windows(2) {
            let (left_position, left_color) = pair[0];
            let (right_position, right_color) = pair[1];

            if position >= left_position && position <= right_position {
                let span = right_position - left_position;

                if span <= f32::EPSILON {
                    return Some(right_color);
                }

                let factor = (position - left_position) / span;

                let mut output = [0_u8; 4];

                for index in 0..4 {
                    let left = f32::from(left_color[index]);
                    let right = f32::from(right_color[index]);
                    let value = left + ((right - left) * factor);

                    output[index] = value.round().clamp(0.0, 255.0) as u8;
                }

                return Some(output);
            }
        }

        Some(last.1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemRegistry {
    by_name: AHashMap<String, ItemDefinition>,
    aliases: AHashMap<String, String>,
}

impl ItemRegistry {
    pub fn from_parts(
        definitions: Vec<ItemDefinition>,
        aliases: Vec<(String, String)>,
    ) -> Result<Self, ProtocolError> {
        let mut registry = Self::default();

        for definition in definitions {
            if registry.by_name.contains_key(&definition.name) {
                return Err(ProtocolError::InvalidItemDef("duplicate item name"));
            }

            registry.by_name.insert(definition.name.clone(), definition);
        }

        for (alias, target) in aliases {
            if !registry.by_name.contains_key(&alias) {
                registry.aliases.insert(alias, target);
            }
        }

        Ok(registry)
    }

    #[must_use]
    pub fn resolve_name<'a>(&'a self, name: &'a str) -> &'a str {
        let mut current = name;

        for _ in 0..=self.aliases.len() {
            if self.by_name.contains_key(current) {
                return current;
            }

            let Some(target) = self.aliases.get(current) else {
                return current;
            };
            current = target;
        }

        name
    }

    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&ItemDefinition> {
        let mut current = name;

        for _ in 0..=self.aliases.len() {
            if let Some(definition) = self.by_name.get(current) {
                return Some(definition);
            }

            current = self.aliases.get(current)?;
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &ItemDefinition> {
        self.by_name.values()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    #[must_use]
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

fn read_utf8_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    let value = std::str::from_utf8(bytes)
        .map_err(|_| ProtocolError::InvalidItemDef("invalid UTF-8 string"))?;

    Ok(value.to_owned())
}

fn empty_simple_sound_spec() -> SimpleSoundSpec {
    SimpleSoundSpec {
        name: String::new(),
        gain: 1.0,
        pitch: 1.0,
        fade: 0.0,
    }
}

fn decode_simple_sound_spec(
    reader: &mut PacketReader<'_>,
) -> Result<SimpleSoundSpec, ProtocolError> {
    Ok(SimpleSoundSpec {
        name: read_utf8_string16(reader)?,
        gain: reader.read_f32()?,
        pitch: reader.read_f32()?,
        fade: reader.read_f32()?,
    })
}

fn decode_tool_capabilities(
    data: &[u8],
    _protocol_version: u16,
) -> Result<ToolCapabilities, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version < 4 {
        return Err(ProtocolError::InvalidItemDef(
            "unsupported ToolCapabilities version",
        ));
    }

    let full_punch_interval = reader.read_f32()?;
    let max_drop_level = reader.read_i16()?;

    let groupcaps_count = usize::try_from(reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidItemDef("invalid groupcaps count"))?;

    if groupcaps_count > reader.remaining() / 10 {
        return Err(ProtocolError::InvalidItemDef("invalid groupcaps count"));
    }

    let mut groupcaps = Vec::with_capacity(groupcaps_count);
    let mut seen_groupcaps = AHashMap::new();

    for _ in 0..groupcaps_count {
        let name = read_utf8_string16(&mut reader)?;

        if seen_groupcaps.insert(name.clone(), ()).is_some() {
            return Err(ProtocolError::InvalidItemDef("duplicate tool groupcap"));
        }

        let uses = reader.read_i16()?;
        let max_level = reader.read_i16()?;

        let times_count = usize::try_from(reader.read_u32()?)
            .map_err(|_| ProtocolError::InvalidItemDef("invalid tool times count"))?;

        if times_count > reader.remaining() / 6 {
            return Err(ProtocolError::InvalidItemDef("invalid tool times count"));
        }

        let mut times = Vec::with_capacity(times_count);
        let mut seen_ratings = AHashMap::new();

        for _ in 0..times_count {
            let rating = reader.read_i16()?;
            let time = reader.read_f32()?;

            if seen_ratings.insert(rating, ()).is_some() {
                return Err(ProtocolError::InvalidItemDef("duplicate tool rating"));
            }

            times.push((rating, time));
        }

        groupcaps.push(ToolGroupCapability {
            name,
            uses,
            max_level,
            times,
        });
    }

    let damage_count = usize::try_from(reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidItemDef("invalid damage group count"))?;

    if damage_count > reader.remaining() / 4 {
        return Err(ProtocolError::InvalidItemDef("invalid damage group count"));
    }

    let mut damage_groups = Vec::with_capacity(damage_count);
    let mut seen_damage_groups = AHashMap::new();

    for _ in 0..damage_count {
        let name = read_utf8_string16(&mut reader)?;
        let value = reader.read_i16()?;

        if seen_damage_groups.insert(name.clone(), ()).is_some() {
            return Err(ProtocolError::InvalidItemDef("duplicate damage group"));
        }

        damage_groups.push((name, value));
    }

    let punch_attack_uses = if version >= 5 {
        Some(reader.read_u16()?)
    } else {
        None
    };

    Ok(ToolCapabilities {
        full_punch_interval,
        max_drop_level,
        groupcaps,
        damage_groups,
        punch_attack_uses,
    })
}

fn decode_item_groups(reader: &mut PacketReader<'_>) -> Result<Vec<(String, i16)>, ProtocolError> {
    let count = usize::from(reader.read_u16()?);

    if count > reader.remaining() / 4 {
        return Err(ProtocolError::InvalidItemDef("invalid item group count"));
    }

    let mut groups = Vec::with_capacity(count);
    let mut seen_groups = AHashMap::new();

    for _ in 0..count {
        let name = read_utf8_string16(reader)?;
        let value = reader.read_i16()?;

        if seen_groups.insert(name.clone(), ()).is_some() {
            return Err(ProtocolError::InvalidItemDef("duplicate item group"));
        }

        groups.push((name, value));
    }

    Ok(groups)
}

fn decode_pointability_map(
    reader: &mut PacketReader<'_>,
) -> Result<Vec<(String, PointabilityType)>, ProtocolError> {
    let count = usize::try_from(reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidItemDef("invalid pointability map size"))?;

    if count > reader.remaining() / 3 {
        return Err(ProtocolError::InvalidItemDef(
            "invalid pointability map size",
        ));
    }

    let mut entries = Vec::with_capacity(count);
    let mut seen = AHashMap::new();

    for _ in 0..count {
        let name = read_utf8_string16(reader)?;

        if name.is_empty() {
            return Err(ProtocolError::InvalidItemDef(
                "empty pointability rule name",
            ));
        }

        let raw_type = reader.read_u8()?;

        let pointability =
            PointabilityType::from_repr(raw_type).unwrap_or(PointabilityType::Pointable);

        if seen.insert(name.clone(), ()).is_some() {
            return Err(ProtocolError::InvalidItemDef("duplicate pointability rule"));
        }

        entries.push((name, pointability));
    }

    Ok(entries)
}

fn decode_pointabilities(data: &[u8]) -> Result<Pointabilities, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version != 0 {
        return Err(ProtocolError::InvalidItemDef(
            "unsupported pointabilities version",
        ));
    }

    let nodes = decode_pointability_map(&mut reader)?;
    let node_groups = decode_pointability_map(&mut reader)?;
    let objects = decode_pointability_map(&mut reader)?;
    let object_groups = decode_pointability_map(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidItemDef(
            "trailing bytes in pointabilities",
        ));
    }

    Ok(Pointabilities {
        nodes,
        node_groups,
        objects,
        object_groups,
    })
}

fn decode_wear_bar_params(reader: &mut PacketReader<'_>) -> Result<WearBarParams, ProtocolError> {
    let version = reader.read_u8()?;

    if version > 1 {
        return Err(ProtocolError::InvalidItemDef(
            "unsupported wear bar params version",
        ));
    }

    let blend = WearBarBlendMode::from_repr(reader.read_u8()?)
        .ok_or(ProtocolError::InvalidItemDef("invalid wear bar blend mode"))?;
    let count = usize::from(reader.read_u16()?);

    if count == 0 || count > reader.remaining() / 8 {
        return Err(ProtocolError::InvalidItemDef(
            "invalid wear bar color stop count",
        ));
    }

    let mut color_stops = Vec::with_capacity(count);
    let mut previous_position = None;

    for _ in 0..count {
        let position = reader.read_f32()?;

        if !position.is_finite() || !(0.0..=1.0).contains(&position) {
            return Err(ProtocolError::InvalidItemDef(
                "invalid wear bar color stop position",
            ));
        }

        if let Some(previous) = previous_position
            && position <= previous
        {
            return Err(ProtocolError::InvalidItemDef(
                "wear bar color stops are not strictly ordered",
            ));
        }

        let color_bytes = reader.read_bytes(4)?;

        let color = [
            color_bytes[0],
            color_bytes[1],
            color_bytes[2],
            color_bytes[3],
        ];

        color_stops.push((position, color));
        previous_position = Some(position);
    }

    Ok(WearBarParams { color_stops, blend })
}

fn decode_item_definition_extensions(
    reader: &mut PacketReader<'_>,
    protocol_version: u16,
) -> Result<ItemDefinitionExtensions, ProtocolError> {
    let mut extensions = ItemDefinitionExtensions {
        short_description: String::new(),
        place_param2: None,
        sound_use: empty_simple_sound_spec(),
        sound_use_air: empty_simple_sound_spec(),
        wallmounted_rotate_vertical: false,
        touch_interaction: [2; 3],
        pointabilities: None,
        wear_bar_params: None,
    };

    if reader.is_empty() {
        return Ok(extensions);
    }

    extensions.short_description = read_utf8_string16(reader)?;

    if reader.is_empty() {
        return Ok(extensions);
    }

    if protocol_version <= 43 {
        let value = reader.read_u8()?;
        extensions.place_param2 = (value != 0).then_some(value);
    }

    if reader.is_empty() {
        return Ok(extensions);
    }

    extensions.sound_use = decode_simple_sound_spec(reader)?;
    extensions.sound_use_air = decode_simple_sound_spec(reader)?;

    if reader.is_empty() {
        return Ok(extensions);
    }

    if reader.read_u8()? != 0 {
        extensions.place_param2 = Some(reader.read_u8()?);
    }

    if reader.is_empty() {
        return Ok(extensions);
    }

    extensions.wallmounted_rotate_vertical = reader.read_u8()? != 0;
    extensions.touch_interaction = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];

    let pointabilities_data = reader.read_string16()?;

    if !pointabilities_data.is_empty() {
        extensions.pointabilities = Some(decode_pointabilities(pointabilities_data)?);
    }

    if !reader.is_empty() && reader.read_u8()? != 0 {
        extensions.wear_bar_params = Some(decode_wear_bar_params(reader)?);
    }

    Ok(extensions)
}

fn decode_item_image_def(
    reader: &mut PacketReader<'_>,
    protocol_version: u16,
) -> Result<ItemImageDef, ProtocolError> {
    let name = read_utf8_string16(reader)?;

    let animation = if protocol_version < 51 {
        TileAnimation::None
    } else {
        match reader.read_u8()? {
            0 => TileAnimation::None,

            1 => TileAnimation::VerticalFrames {
                aspect_w: reader.read_u16()?,
                aspect_h: reader.read_u16()?,
                length: reader.read_f32()?,
            },

            2 => TileAnimation::Sheet2d {
                frames_w: reader.read_u8()?,
                frames_h: reader.read_u8()?,
                frame_length: reader.read_f32()?,
            },

            _ => TileAnimation::None,
        }
    };

    Ok(ItemImageDef { name, animation })
}

pub type DecodedItemDefinitions = (Vec<ItemDefinition>, Vec<(String, String)>);

fn encode_simple_sound_spec(
    writer: &mut PacketWriter,
    sound: &SimpleSoundSpec,
) -> Result<(), ProtocolError> {
    if !sound.gain.is_finite() || !sound.pitch.is_finite() || !sound.fade.is_finite() {
        return Err(ProtocolError::InvalidItemDef("invalid sound parameters"));
    }
    writer.write_string16(&sound.name)?;
    writer.write_f32(sound.gain);
    writer.write_f32(sound.pitch);
    writer.write_f32(sound.fade);
    Ok(())
}

fn encode_item_image_def(
    writer: &mut PacketWriter,
    image: &ItemImageDef,
    protocol_version: u16,
) -> Result<(), ProtocolError> {
    let legacy_name;
    let image_name = if protocol_version < 51 {
        legacy_name = match &image.animation {
            TileAnimation::None => image.name.clone(),
            TileAnimation::VerticalFrames {
                aspect_w, aspect_h, ..
            } => {
                if *aspect_w == 0 || *aspect_h == 0 {
                    return Err(ProtocolError::InvalidItemDef(
                        "invalid vertical image animation",
                    ));
                }
                let escaped_name = image
                    .name
                    .replace('\\', "\\\\")
                    .replace(':', "\\:")
                    .replace('^', "\\^");
                format!("[combine:{aspect_w}x{aspect_h}:0,0={escaped_name}")
            }
            TileAnimation::Sheet2d {
                frames_w, frames_h, ..
            } => {
                if *frames_w == 0 || *frames_h == 0 {
                    return Err(ProtocolError::InvalidItemDef(
                        "invalid sheet image animation",
                    ));
                }
                format!("{}^[sheet:{frames_w}x{frames_h}:0,0", image.name)
            }
        };
        &legacy_name
    } else {
        &image.name
    };
    writer.write_string16(image_name)?;
    if protocol_version >= 51 {
        match &image.animation {
            TileAnimation::None => writer.write_u8(0),
            TileAnimation::VerticalFrames {
                aspect_w,
                aspect_h,
                length,
            } => {
                if *aspect_w == 0 || *aspect_h == 0 || !length.is_finite() || *length <= 0.0 {
                    return Err(ProtocolError::InvalidItemDef(
                        "invalid vertical image animation",
                    ));
                }
                writer.write_u8(1);
                writer.write_u16(*aspect_w);
                writer.write_u16(*aspect_h);
                writer.write_f32(*length);
            }
            TileAnimation::Sheet2d {
                frames_w,
                frames_h,
                frame_length,
            } => {
                if *frames_w == 0
                    || *frames_h == 0
                    || !frame_length.is_finite()
                    || *frame_length <= 0.0
                {
                    return Err(ProtocolError::InvalidItemDef(
                        "invalid sheet image animation",
                    ));
                }
                writer.write_u8(2);
                writer.write_u8(*frames_w);
                writer.write_u8(*frames_h);
                writer.write_f32(*frame_length);
            }
        }
    }
    Ok(())
}

fn encode_tool_capabilities(
    value: &ToolCapabilities,
    protocol_version: u16,
) -> Result<Bytes, ProtocolError> {
    if !value.full_punch_interval.is_finite() || value.full_punch_interval < 0.0 {
        return Err(ProtocolError::InvalidItemDef("invalid tool punch interval"));
    }
    let mut writer = PacketWriter::new();
    let capabilities_version = if protocol_version >= 38 { 5 } else { 4 };
    writer.write_u8(capabilities_version);
    writer.write_f32(value.full_punch_interval);
    writer.write_i16(value.max_drop_level);
    writer.write_u32(
        u32::try_from(value.groupcaps.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many tool group capabilities"))?,
    );
    for group in &value.groupcaps {
        writer.write_string16(&group.name)?;
        writer.write_i16(group.uses);
        writer.write_i16(group.max_level);
        writer.write_u32(
            u32::try_from(group.times.len())
                .map_err(|_| ProtocolError::InvalidItemDef("too many tool group times"))?,
        );
        for (rating, time) in &group.times {
            if !time.is_finite() || *time < 0.0 {
                return Err(ProtocolError::InvalidItemDef("invalid tool group time"));
            }
            writer.write_i16(*rating);
            writer.write_f32(*time);
        }
    }
    writer.write_u32(
        u32::try_from(value.damage_groups.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many tool damage groups"))?,
    );
    for (name, damage) in &value.damage_groups {
        writer.write_string16(name)?;
        writer.write_i16(*damage);
    }
    if capabilities_version >= 5 {
        writer.write_u16(value.punch_attack_uses.unwrap_or_default());
    }
    Ok(writer.into_inner())
}

fn encode_pointability_map(
    writer: &mut PacketWriter,
    entries: &[(String, PointabilityType)],
) -> Result<(), ProtocolError> {
    writer.write_u32(
        u32::try_from(entries.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many pointability entries"))?,
    );
    for (name, pointability) in entries {
        writer.write_string16(name)?;
        writer.write_u8(u8::from(*pointability));
    }
    Ok(())
}

fn encode_pointabilities(value: &Pointabilities) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();
    writer.write_u8(0);
    encode_pointability_map(&mut writer, &value.nodes)?;
    encode_pointability_map(&mut writer, &value.node_groups)?;
    encode_pointability_map(&mut writer, &value.objects)?;
    encode_pointability_map(&mut writer, &value.object_groups)?;
    Ok(writer.into_inner())
}

fn encode_wear_bar_params(
    writer: &mut PacketWriter,
    value: &WearBarParams,
) -> Result<(), ProtocolError> {
    if value.color_stops.is_empty() {
        return Err(ProtocolError::InvalidItemDef(
            "invalid wear bar color stop count",
        ));
    }
    writer.write_u8(1);
    writer.write_u8(u8::from(value.blend));
    writer.write_u16(
        u16::try_from(value.color_stops.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many wear bar color stops"))?,
    );
    let mut previous = None;
    for (position, color) in &value.color_stops {
        if !position.is_finite()
            || !(0.0..=1.0).contains(position)
            || previous.is_some_and(|last| *position <= last)
        {
            return Err(ProtocolError::InvalidItemDef("invalid wear bar color stop"));
        }
        writer.write_f32(*position);
        writer.write_bytes(color);
        previous = Some(*position);
    }
    Ok(())
}

fn encode_item_definition(
    value: &ItemDefinition,
    protocol_version: u16,
) -> Result<Bytes, ProtocolError> {
    if value.item_type > 3
        || value.stack_max == 0
        || !value.wield_scale.is_finite()
        || !value.range.is_finite()
    {
        return Err(ProtocolError::InvalidItemDef(
            "invalid item definition values",
        ));
    }

    let mut writer = PacketWriter::new();
    writer.write_u8(ITEM_DEFINITION_VERSION);
    writer.write_u8(value.item_type);
    writer.write_string16(&value.name)?;
    writer.write_string16(&value.description)?;
    encode_item_image_def(&mut writer, &value.inventory_image, protocol_version)?;
    encode_item_image_def(&mut writer, &value.wield_image, protocol_version)?;
    writer.write_f32(value.wield_scale.x);
    writer.write_f32(value.wield_scale.y);
    writer.write_f32(value.wield_scale.z);
    writer.write_u16(value.stack_max);
    writer.write_u8(u8::from(value.usable));
    writer.write_u8(u8::from(value.liquids_pointable));

    if let Some(capabilities) = &value.tool_capabilities {
        let encoded = encode_tool_capabilities(capabilities, protocol_version)?;
        writer.write_binary_string16(&encoded)?;
    } else {
        writer.write_u16(0);
    }

    writer.write_u16(
        u16::try_from(value.groups.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many item groups"))?,
    );
    for (name, rating) in &value.groups {
        writer.write_string16(name)?;
        writer.write_i16(*rating);
    }
    writer.write_string16(&value.node_placement_prediction)?;
    encode_simple_sound_spec(&mut writer, &value.sound_place)?;
    encode_simple_sound_spec(&mut writer, &value.sound_place_failed)?;
    writer.write_f32(value.range);
    writer.write_string16(&value.palette_image)?;
    writer.write_bytes(&value.color);
    encode_item_image_def(&mut writer, &value.inventory_overlay, protocol_version)?;
    encode_item_image_def(&mut writer, &value.wield_overlay, protocol_version)?;

    writer.write_string16(&value.short_description)?;
    if protocol_version <= 43 {
        writer.write_u8(value.place_param2.unwrap_or(0));
    }
    encode_simple_sound_spec(&mut writer, &value.sound_use)?;
    encode_simple_sound_spec(&mut writer, &value.sound_use_air)?;
    if let Some(place_param2) = value.place_param2 {
        writer.write_u8(1);
        writer.write_u8(place_param2);
    } else {
        writer.write_u8(0);
    }
    writer.write_u8(u8::from(value.wallmounted_rotate_vertical));
    writer.write_bytes(&value.touch_interaction);
    if let Some(pointabilities) = &value.pointabilities {
        let encoded = encode_pointabilities(pointabilities)?;
        writer.write_binary_string16(&encoded)?;
    } else {
        writer.write_u16(0);
    }
    if let Some(wear_bar) = &value.wear_bar_params {
        writer.write_u8(1);
        encode_wear_bar_params(&mut writer, wear_bar)?;
    } else {
        writer.write_u8(0);
    }
    Ok(writer.into_inner())
}

pub fn encode_item_def(
    definitions: &[ItemDefinition],
    aliases: &[(String, String)],
    protocol_version: u16,
) -> Result<Bytes, ProtocolError> {
    let mut manager = PacketWriter::with_capacity(definitions.len().saturating_mul(128));
    manager.write_u8(ITEMDEF_MANAGER_VERSION);
    manager.write_u16(
        u16::try_from(definitions.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many item definitions"))?,
    );
    let mut names = ahash::AHashSet::with_capacity(definitions.len());
    for definition in definitions {
        if !names.insert(definition.name.as_str()) {
            return Err(ProtocolError::InvalidItemDef(
                "duplicate item definition name",
            ));
        }
        let encoded = encode_item_definition(definition, protocol_version)?;
        manager.write_binary_string16(&encoded)?;
    }
    manager.write_u16(
        u16::try_from(aliases.len())
            .map_err(|_| ProtocolError::InvalidItemDef("too many item aliases"))?,
    );
    for (alias, target) in aliases {
        manager.write_string16(alias)?;
        manager.write_string16(target)?;
    }
    let manager = manager.into_inner();
    if manager.len() > MAX_ITEMDEF_SIZE {
        return Err(ProtocolError::InvalidItemDef(
            "item definition data exceeds size limit",
        ));
    }
    let compressed = if protocol_version >= crate::protocol::compression::ZSTD_PROTOCOL_VERSION {
        zstd::encode_all(Cursor::new(manager), 3)
            .map_err(|_| ProtocolError::InvalidItemDef("item definition compression failed"))?
    } else {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&manager)
            .map_err(|_| ProtocolError::InvalidItemDef("item definition compression failed"))?;
        encoder
            .finish()
            .map_err(|_| ProtocolError::InvalidItemDef("item definition compression failed"))?
    };
    if compressed.len() > MAX_ITEMDEF_SIZE {
        return Err(ProtocolError::InvalidItemDef(
            "compressed item definitions exceed size limit",
        ));
    }
    let mut packet = PacketWriter::with_capacity(compressed.len().saturating_add(4));
    packet.write_u32(
        u32::try_from(compressed.len()).map_err(|_| {
            ProtocolError::InvalidItemDef("compressed item definitions are too large")
        })?,
    );
    packet.write_bytes(&compressed);
    Ok(packet.into_inner())
}

pub fn decode_item_def(
    data: &[u8],
    protocol_version: u16,
) -> Result<DecodedItemDefinitions, ProtocolError> {
    let mut packet_reader = PacketReader::new(data);

    let compressed_length = usize::try_from(packet_reader.read_u32()?)
        .map_err(|_| ProtocolError::InvalidItemDef("invalid blob length"))?;

    if compressed_length > MAX_ITEMDEF_SIZE {
        return Err(ProtocolError::InvalidItemDef(
            "compressed blob exceeds size limit",
        ));
    }

    let compressed_data = packet_reader.read_bytes(compressed_length)?;

    if !packet_reader.is_empty() {
        return Err(ProtocolError::InvalidItemDef(
            "unexpected bytes after compressed blob",
        ));
    }

    let decompressed = decompress_limited(compressed_data, MAX_ITEMDEF_SIZE, protocol_version)
        .map_err(ProtocolError::InvalidItemDef)?;
    let mut reader = PacketReader::new(&decompressed);

    let manager_version = reader.read_u8()?;

    if manager_version != ITEMDEF_MANAGER_VERSION {
        return Err(ProtocolError::InvalidItemDef(
            "unsupported item definition manager version",
        ));
    }

    let definition_count = usize::from(reader.read_u16()?);

    if definition_count > reader.remaining() / 2 {
        return Err(ProtocolError::InvalidItemDef(
            "item definition count exceeds packet size",
        ));
    }

    let mut definitions = Vec::with_capacity(definition_count);

    for _ in 0..definition_count {
        let serialized_definition = reader.read_string16()?;
        let mut definition_reader = PacketReader::new(serialized_definition);

        let definition_version = definition_reader.read_u8()?;

        if definition_version < 6 {
            return Err(ProtocolError::InvalidItemDef(
                "unsupported item definition version",
            ));
        }

        let mut item_type = definition_reader.read_u8()?;

        if item_type > 3 {
            item_type = 0;
        }

        let name = read_utf8_string16(&mut definition_reader)?;
        let description = read_utf8_string16(&mut definition_reader)?;

        let inventory_image = decode_item_image_def(&mut definition_reader, protocol_version)?;

        let wield_image = decode_item_image_def(&mut definition_reader, protocol_version)?;

        let wield_scale = Vec3::new(
            definition_reader.read_f32()?,
            definition_reader.read_f32()?,
            definition_reader.read_f32()?,
        );

        let stack_max = definition_reader.read_u16()?;
        let usable = definition_reader.read_u8()? != 0;
        let liquids_pointable = definition_reader.read_u8()? != 0;

        let tool_capabilities_data = definition_reader.read_string16()?;

        let tool_capabilities = if tool_capabilities_data.is_empty() {
            None
        } else {
            Some(decode_tool_capabilities(
                tool_capabilities_data,
                protocol_version,
            )?)
        };

        let groups = decode_item_groups(&mut definition_reader)?;

        let node_placement_prediction = read_utf8_string16(&mut definition_reader)?;

        let sound_place = decode_simple_sound_spec(&mut definition_reader)?;

        let sound_place_failed = decode_simple_sound_spec(&mut definition_reader)?;

        let range = definition_reader.read_f32()?;

        let palette_image = read_utf8_string16(&mut definition_reader)?;

        let color_bytes = definition_reader.read_bytes(4)?;
        let color = [
            color_bytes[0],
            color_bytes[1],
            color_bytes[2],
            color_bytes[3],
        ];

        let inventory_overlay = decode_item_image_def(&mut definition_reader, protocol_version)?;

        let wield_overlay = decode_item_image_def(&mut definition_reader, protocol_version)?;

        let extensions =
            decode_item_definition_extensions(&mut definition_reader, protocol_version)?;

        definitions.push(ItemDefinition {
            item_type,
            name,
            description,
            short_description: extensions.short_description,

            inventory_image,
            inventory_overlay,
            wield_image,
            wield_overlay,

            wield_scale,
            stack_max,
            usable,
            liquids_pointable,

            tool_capabilities,
            groups,

            node_placement_prediction,
            sound_place,
            sound_place_failed,
            sound_use: extensions.sound_use,
            sound_use_air: extensions.sound_use_air,

            range,
            palette_image,
            color,

            place_param2: extensions.place_param2,
            wallmounted_rotate_vertical: extensions.wallmounted_rotate_vertical,
            touch_interaction: extensions.touch_interaction,

            pointabilities: extensions.pointabilities,
            wear_bar_params: extensions.wear_bar_params,
        });
    }

    let alias_count = usize::from(reader.read_u16()?);

    if alias_count > reader.remaining() / 4 {
        return Err(ProtocolError::InvalidItemDef(
            "item alias count exceeds packet size",
        ));
    }

    let mut aliases = Vec::with_capacity(alias_count);

    for _ in 0..alias_count {
        let alias = read_utf8_string16(&mut reader)?;
        let target = read_utf8_string16(&mut reader)?;

        aliases.push((alias, target));
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidItemDef(
            "unexpected bytes after item definitions",
        ));
    }

    Ok((definitions, aliases))
}
