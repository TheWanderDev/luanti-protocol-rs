use bytes::Bytes;
use glam::I16Vec3;

use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::writer::PacketWriter;

pub fn encode_got_blocks(positions: &[I16Vec3]) -> Result<Bytes, ProtocolError> {
    encode_block_positions(ToServerCommand::GotBlocks, positions)
}

pub fn encode_deleted_blocks(positions: &[I16Vec3]) -> Result<Bytes, ProtocolError> {
    encode_block_positions(ToServerCommand::DeletedBlocks, positions)
}

fn encode_block_positions(
    command: ToServerCommand,
    positions: &[I16Vec3],
) -> Result<Bytes, ProtocolError> {
    let count = u8::try_from(positions.len()).map_err(|_| ProtocolError::InvalidBlockBatch)?;
    let capacity = positions.len().saturating_mul(6).saturating_add(3);
    let mut writer = PacketWriter::with_capacity(capacity);
    writer.write_u16(u16::from(command));
    writer.write_u8(count);

    for position in positions {
        writer.write_i16(position.x);
        writer.write_i16(position.y);
        writer.write_i16(position.z);
    }

    Ok(writer.into_inner())
}

pub fn encode_removed_sounds(sound_ids: &[i32]) -> Result<Bytes, ProtocolError> {
    let count = u16::try_from(sound_ids.len())
        .map_err(|_| ProtocolError::InvalidSound("too many removed sound IDs"))?;
    let capacity = sound_ids.len().saturating_mul(4).saturating_add(4);
    let mut writer = PacketWriter::with_capacity(capacity);
    writer.write_u16(u16::from(ToServerCommand::RemovedSounds));
    writer.write_u16(count);

    for sound_id in sound_ids {
        writer.write_i32(*sound_id);
    }

    Ok(writer.into_inner())
}

#[must_use]
pub fn encode_damage(damage: u16) -> Bytes {
    let mut writer = PacketWriter::with_capacity(4);
    writer.write_u16(u16::from(ToServerCommand::Damage));
    writer.write_u16(damage);
    writer.into_inner()
}

#[must_use]
pub fn encode_respawn_legacy() -> Bytes {
    let mut writer = PacketWriter::with_capacity(2);
    writer.write_u16(u16::from(ToServerCommand::RespawnLegacy));
    writer.into_inner()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientInfoUpdate {
    pub render_target_width: u32,
    pub render_target_height: u32,
    pub gui_scaling: f32,
    pub hud_scaling: f32,
    pub formspec_width: f32,
    pub formspec_height: f32,
    pub touch_controls: bool,
}

pub fn encode_update_client_info(update: ClientInfoUpdate) -> Result<Bytes, ProtocolError> {
    let values = [
        update.gui_scaling,
        update.hud_scaling,
        update.formspec_width,
        update.formspec_height,
    ];
    if !values.iter().all(|value| value.is_finite()) {
        return Err(ProtocolError::InvalidPlayerValue(
            "client display information",
        ));
    }

    let mut writer = PacketWriter::with_capacity(27);
    writer.write_u16(u16::from(ToServerCommand::UpdateClientInfo));
    writer.write_u32(update.render_target_width);
    writer.write_u32(update.render_target_height);
    writer.write_f32(update.gui_scaling);
    writer.write_f32(update.hud_scaling);
    writer.write_f32(update.formspec_width);
    writer.write_f32(update.formspec_height);
    writer.write_u8(u8::from(update.touch_controls));
    Ok(writer.into_inner())
}
