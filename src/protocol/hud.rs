use glam::{IVec2, Vec2, Vec3};

use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;
use crate::protocol::vectors::{read_vec2_i32_scaled, read_vec3_i32_scaled};

const PROTOCOL_HUD_SIZE_FLOAT: u16 = 52;
const HUD_VECTOR_SCALE: f32 = 1_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudChangeStat {
    Position,
    Name,
    Scale,
    Text,
    Number,
    Item,
    Direction,
    Alignment,
    Offset,
    WorldPosition,
    Size,
    ZIndex,
    Text2,
    Style,
    Hideable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HudChangeValue {
    Vec2(Vec2),
    Vec3(Vec3),
    String(String),
    Integer(u32),
    SignedInteger(i16),
    Size(HudSize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudChange {
    pub id: u32,
    pub stat: HudChangeStat,
    pub value: HudChangeValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HudSize {
    Integer(IVec2),
    Float(Vec2),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudAdd {
    pub id: u32,
    pub hud_type: u8,
    pub position: Vec2,
    pub name: String,
    pub scale: Vec2,
    pub text: String,
    pub number: u32,
    pub item: u32,
    pub dir: u32,
    pub align: Vec2,
    pub offset: Vec2,
    pub world_position: Vec3,
    pub size: HudSize,
    pub z_index: i16,
    pub text2: String,
    pub style: u32,
    pub hideable: bool,
}

fn read_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec()).map_err(|_| ProtocolError::InvalidHud("string is not UTF-8"))
}

pub(crate) fn decode_hud_add(data: &[u8], protocol_version: u16) -> Result<HudAdd, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let id = reader.read_u32()?;
    let hud_type = reader.read_u8()?;
    let position = read_vec2_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?;
    let name = read_string16(&mut reader)?;
    let scale = read_vec2_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?;
    let text = read_string16(&mut reader)?;

    let number = reader.read_u32()?;
    let item = reader.read_u32()?;
    let dir = reader.read_u32()?;

    let align = read_vec2_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?;
    let offset = read_vec2_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?;
    let world_position = read_vec3_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?;

    let size = if protocol_version >= PROTOCOL_HUD_SIZE_FLOAT {
        HudSize::Float(Vec2::new(reader.read_f32()?, reader.read_f32()?))
    } else {
        HudSize::Integer(IVec2::new(reader.read_i32()?, reader.read_i32()?))
    };
    let mut z_index = 0;
    let mut text2 = String::new();
    let mut style = 0;
    let mut flags = 1_u8;

    if !reader.is_empty() {
        z_index = reader.read_i16()?;
    }
    if !reader.is_empty() {
        text2 = read_string16(&mut reader)?;
    }
    if !reader.is_empty() {
        style = reader.read_u32()?;
    }
    if !reader.is_empty() {
        flags = reader.read_u8()?;
    }

    let hideable = flags & 1 != 0;
    if !reader.is_empty() {
        let remaining = reader.remaining();
        let consumed = data.len() - remaining;

        tracing::warn!(
            protocol_version,
            payload_len = data.len(),
            consumed,
            remaining,
            trailing_bytes = ?&data[consumed..],
            "HUDADD contains unread bytes"
        );

        return Err(ProtocolError::InvalidHud("trailing packet data"));
    }

    Ok(HudAdd {
        id,
        hud_type,
        position,
        name,
        scale,
        text,
        number,
        item,
        dir,
        align,
        offset,
        world_position,
        size,
        z_index,
        text2,
        style,
        hideable,
    })
}

pub(crate) fn decode_hud_remove(data: &[u8]) -> Result<u32, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let id = reader.read_u32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidHud("trailing packet data"));
    }

    Ok(id)
}

pub(crate) fn decode_hud_change(
    data: &[u8],
    protocol_version: u16,
) -> Result<HudChange, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let id = reader.read_u32()?;
    let stat_code = reader.read_u8()?;
    let stat = match stat_code {
        0 => HudChangeStat::Position,
        1 => HudChangeStat::Name,
        2 => HudChangeStat::Scale,
        3 => HudChangeStat::Text,
        4 => HudChangeStat::Number,
        5 => HudChangeStat::Item,
        6 => HudChangeStat::Direction,
        7 => HudChangeStat::Alignment,
        8 => HudChangeStat::Offset,
        9 => HudChangeStat::WorldPosition,
        10 => HudChangeStat::Size,
        11 => HudChangeStat::ZIndex,
        12 => HudChangeStat::Text2,
        13 => HudChangeStat::Style,
        14 => HudChangeStat::Hideable,
        _ => return Err(ProtocolError::InvalidHud("unknown HUD change stat")),
    };
    let value = match stat {
        HudChangeStat::Position
        | HudChangeStat::Scale
        | HudChangeStat::Alignment
        | HudChangeStat::Offset => {
            HudChangeValue::Vec2(read_vec2_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?)
        }

        HudChangeStat::Name | HudChangeStat::Text | HudChangeStat::Text2 => {
            HudChangeValue::String(read_string16(&mut reader)?)
        }

        HudChangeStat::WorldPosition => {
            HudChangeValue::Vec3(read_vec3_i32_scaled(&mut reader, HUD_VECTOR_SCALE)?)
        }

        HudChangeStat::Size => {
            let size = if protocol_version >= PROTOCOL_HUD_SIZE_FLOAT {
                HudSize::Float(Vec2::new(reader.read_f32()?, reader.read_f32()?))
            } else {
                HudSize::Integer(IVec2::new(reader.read_i32()?, reader.read_i32()?))
            };

            HudChangeValue::Size(size)
        }

        HudChangeStat::ZIndex => HudChangeValue::SignedInteger(reader.read_i16()?),

        HudChangeStat::Number
        | HudChangeStat::Item
        | HudChangeStat::Direction
        | HudChangeStat::Style
        | HudChangeStat::Hideable => HudChangeValue::Integer(reader.read_u32()?),
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidHud("trailing packet data"));
    }

    Ok(HudChange { id, stat, value })
}
