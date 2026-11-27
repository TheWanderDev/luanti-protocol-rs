use glam::Vec3;

use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;
use crate::protocol::vectors::read_vec3_f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementParameters {
    pub acceleration_default: f32,
    pub acceleration_air: f32,
    pub acceleration_fast: f32,
    pub speed_walk: f32,
    pub speed_crouch: f32,
    pub speed_fast: f32,
    pub speed_climb: f32,
    pub speed_jump: f32,
    pub liquid_fluidity: f32,
    pub liquid_fluidity_smooth: f32,
    pub liquid_sink: f32,
    pub gravity: f32,
}

pub fn decode_movement_parameters(data: &[u8]) -> Result<MovementParameters, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let parameters = MovementParameters {
        acceleration_default: reader.read_f32()?,
        acceleration_air: reader.read_f32()?,
        acceleration_fast: reader.read_f32()?,
        speed_walk: reader.read_f32()?,
        speed_crouch: reader.read_f32()?,
        speed_fast: reader.read_f32()?,
        speed_climb: reader.read_f32()?,
        speed_jump: reader.read_f32()?,
        liquid_fluidity: reader.read_f32()?,
        liquid_fluidity_smooth: reader.read_f32()?,
        liquid_sink: reader.read_f32()?,
        gravity: reader.read_f32()?,
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    let values = [
        parameters.acceleration_default,
        parameters.acceleration_air,
        parameters.acceleration_fast,
        parameters.speed_walk,
        parameters.speed_crouch,
        parameters.speed_fast,
        parameters.speed_climb,
        parameters.speed_jump,
        parameters.liquid_fluidity,
        parameters.liquid_fluidity_smooth,
        parameters.liquid_sink,
        parameters.gravity,
    ];

    for value in &values {
        if !value.is_finite() {
            return Err(ProtocolError::InvalidPlayerValue(
                "invalid movement parameters",
            ));
        }
    }

    Ok(parameters)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeOfDayUpdate {
    pub time: u16,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpUpdate {
    pub hp: u16,
    pub damage_effect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpeedUpdate {
    pub added_velocity: Vec3,
}

pub fn decode_time_of_day(data: &[u8]) -> Result<TimeOfDayUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let time = reader.read_u16()?;
    let speed = reader.read_f32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    if !speed.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid time of day speed",
        ));
    }

    Ok(TimeOfDayUpdate {
        time: time % 24_000,
        speed,
    })
}

pub fn decode_player_hp(data: &[u8]) -> Result<HpUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let hp = reader.read_u16()?;

    let damage_effect = match reader.remaining() {
        0 => true,
        1 => match reader.read_u8()? {
            0 => false,
            1 => true,
            _ => return Err(ProtocolError::InvalidPlayerPacket),
        },
        _ => return Err(ProtocolError::InvalidPlayerPacket),
    };

    Ok(HpUpdate { hp, damage_effect })
}

pub fn decode_player_breath(data: &[u8]) -> Result<u16, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let breath = reader.read_u16()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(breath)
}

pub fn decode_player_speed(data: &[u8]) -> Result<PlayerSpeedUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let added_velocity = read_vec3_f32(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    if !added_velocity.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue("player speed"));
    }

    Ok(PlayerSpeedUpdate { added_velocity })
}
