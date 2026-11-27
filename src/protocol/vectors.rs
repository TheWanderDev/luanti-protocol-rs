use glam::{Vec2, Vec3};

use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;

pub(crate) fn read_vec2_i32_scaled(
    reader: &mut PacketReader<'_>,
    scale: f32,
) -> Result<Vec2, ProtocolError> {
    let x = reader.read_i32()? as f32 / scale;
    let y = reader.read_i32()? as f32 / scale;

    Ok(Vec2::new(x, y))
}

pub(crate) fn read_vec3_i32_scaled(
    reader: &mut PacketReader<'_>,
    scale: f32,
) -> Result<Vec3, ProtocolError> {
    let x = reader.read_i32()? as f32 / scale;
    let y = reader.read_i32()? as f32 / scale;
    let z = reader.read_i32()? as f32 / scale;

    Ok(Vec3::new(x, y, z))
}

pub(crate) fn read_vec3_f32(reader: &mut PacketReader<'_>) -> Result<Vec3, ProtocolError> {
    let x = reader.read_f32()?;
    let y = reader.read_f32()?;
    let z = reader.read_f32()?;

    Ok(Vec3::new(x, y, z))
}
