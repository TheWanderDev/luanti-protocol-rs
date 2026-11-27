use bytes::Bytes;
use glam::{IVec3, Vec3};

use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;

const PLAYER_UPDATE_SCALE: f32 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyDeathscreen {
    pub set_camera_point_target: bool,
    pub camera_point_target: Vec3,
}

pub fn decode_deathscreen_legacy(payload: &[u8]) -> Result<LegacyDeathscreen, ProtocolError> {
    let mut reader = PacketReader::new(payload);
    let set_camera_point_target = match reader.read_u8()? {
        0 => false,
        1 => true,
        _ => return Err(ProtocolError::InvalidPlayerPacket),
    };
    let camera_point_target = Vec3::new(
        reader.read_f1000()?,
        reader.read_f1000()?,
        reader.read_f1000()?,
    );

    if !reader.is_empty() || !camera_point_target.is_finite() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(LegacyDeathscreen {
        set_camera_point_target,
        camera_point_target,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerPlayerMovement {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPositionUpdate {
    pub position: Vec3,
    pub speed: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub keys_pressed: u32,
    pub fov: u8,
    pub wanted_range: u8,
    pub camera_inverted: bool,
    pub movement_speed: f32,
    pub movement_direction: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum InteractAction {
    StartDigging = 0,
    StopDigging = 1,
    DiggingCompleted = 2,
    Place = 3,
    Use = 4,
    Activate = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointedThing {
    Nothing,
    Node { under: IVec3, above: IVec3 },
    Object { object_id: u16 },
}

fn write_i16_value(writer: &mut PacketWriter, value: i32) -> Result<(), ProtocolError> {
    let value =
        i16::try_from(value).map_err(|_| ProtocolError::InvalidPlayerValue("pointed position"))?;

    writer.write_bytes(&value.to_be_bytes());

    Ok(())
}

fn encode_pointed_thing(pointed: PointedThing) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u8(0);

    match pointed {
        PointedThing::Nothing => {
            writer.write_u8(0);
        }
        PointedThing::Node { under, above } => {
            writer.write_u8(1);

            write_i16_value(&mut writer, under.x)?;
            write_i16_value(&mut writer, under.y)?;
            write_i16_value(&mut writer, under.z)?;

            write_i16_value(&mut writer, above.x)?;
            write_i16_value(&mut writer, above.y)?;
            write_i16_value(&mut writer, above.z)?;
        }
        PointedThing::Object { object_id } => {
            writer.write_u8(2);
            writer.write_u16(object_id);
        }
    }

    Ok(writer.into_inner())
}

pub fn encode_interact(
    action: InteractAction,
    wield_index: u16,
    pointed: PointedThing,
    update: &PlayerPositionUpdate,
) -> Result<Bytes, ProtocolError> {
    if !update.movement_speed.is_finite() || !update.movement_direction.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue("movement"));
    }

    let pointed_data = encode_pointed_thing(pointed)?;
    let pointed_length =
        u32::try_from(pointed_data.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::Interact));
    writer.write_u8(u8::from(action));
    writer.write_u16(wield_index);
    writer.write_u32(pointed_length);
    writer.write_bytes(&pointed_data);
    write_player_position_base(&mut writer, update)?;
    writer.write_f32(update.movement_speed);
    writer.write_f32(update.movement_direction);

    Ok(writer.into_inner())
}

#[must_use]
pub fn encode_player_item(index: u16) -> Bytes {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::PlayerItem));
    writer.write_u16(index);

    writer.into_inner()
}

fn write_scaled_value(
    writer: &mut PacketWriter,
    value: f32,
    scale: f32,
    field: &'static str,
) -> Result<(), ProtocolError> {
    if !value.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(field));
    }

    let scaled = value * scale;

    if !scaled.is_finite() || scaled < i32::MIN as f32 || scaled >= 2_147_483_648.0 {
        return Err(ProtocolError::InvalidPlayerValue(field));
    }

    writer.write_i32(scaled as i32);

    Ok(())
}

fn write_scaled_vec3(
    writer: &mut PacketWriter,
    value: Vec3,
    scale: f32,
    field: &'static str,
) -> Result<(), ProtocolError> {
    write_scaled_value(writer, value.x, scale, field)?;
    write_scaled_value(writer, value.y, scale, field)?;
    write_scaled_value(writer, value.z, scale, field)?;

    Ok(())
}

pub fn decode_move_player(payload: &[u8]) -> Result<ServerPlayerMovement, ProtocolError> {
    let mut reader = PacketReader::new(payload);

    let position = Vec3::new(reader.read_f32()?, reader.read_f32()?, reader.read_f32()?);

    let pitch = -reader.read_f32()?.to_radians();
    let yaw = reader.read_f32()?.to_radians();

    if !reader.is_empty() || !position.is_finite() || !pitch.is_finite() || !yaw.is_finite() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(ServerPlayerMovement {
        position,
        pitch,
        yaw,
    })
}

fn write_player_position_base(
    writer: &mut PacketWriter,
    update: &PlayerPositionUpdate,
) -> Result<(), ProtocolError> {
    write_scaled_vec3(writer, update.position, PLAYER_UPDATE_SCALE, "position")?;
    write_scaled_vec3(writer, update.speed, PLAYER_UPDATE_SCALE, "speed")?;
    write_scaled_value(
        writer,
        -update.pitch.to_degrees(),
        PLAYER_UPDATE_SCALE,
        "pitch",
    )?;
    write_scaled_value(writer, update.yaw.to_degrees(), PLAYER_UPDATE_SCALE, "yaw")?;

    writer.write_u32(update.keys_pressed);
    writer.write_u8(update.fov);
    writer.write_u8(update.wanted_range);
    writer.write_u8(u8::from(update.camera_inverted));

    Ok(())
}

pub fn encode_player_position(update: &PlayerPositionUpdate) -> Result<Bytes, ProtocolError> {
    if !update.movement_speed.is_finite() || !update.movement_direction.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue("movement"));
    }

    let mut writer = PacketWriter::new();
    writer.write_u16(u16::from(ToServerCommand::PlayerPosition));
    write_player_position_base(&mut writer, update)?;
    writer.write_f32(update.movement_speed);
    writer.write_f32(update.movement_direction);
    Ok(writer.into_inner())
}
