use glam::I16Vec3;

use crate::protocol::client_commands::{
    ClientInfoUpdate, encode_damage, encode_deleted_blocks, encode_got_blocks,
    encode_removed_sounds, encode_respawn_legacy, encode_update_client_info,
};
use crate::protocol::commands::{
    TOSERVER_DAMAGE, TOSERVER_DELETEDBLOCKS, TOSERVER_GOTBLOCKS, TOSERVER_REMOVED_SOUNDS,
    TOSERVER_RESPAWN_LEGACY, TOSERVER_UPDATE_CLIENT_INFO,
};
use crate::protocol::messages::{EncodeMessage, Message};

/// Acknowledges map blocks received from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GotBlocks {
    /// Positions of received map blocks.
    pub positions: Vec<I16Vec3>,
}

impl Message for GotBlocks {
    const COMMAND_ID: u16 = TOSERVER_GOTBLOCKS;
}

impl EncodeMessage for GotBlocks {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_got_blocks(&self.positions)
    }
}

/// Reports map blocks that are no longer retained by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedBlocks {
    /// Positions of released map blocks.
    pub positions: Vec<I16Vec3>,
}

impl Message for DeletedBlocks {
    const COMMAND_ID: u16 = TOSERVER_DELETEDBLOCKS;
}

impl EncodeMessage for DeletedBlocks {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_deleted_blocks(&self.positions)
    }
}

/// Reports server-originated sounds that have stopped playing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedSounds {
    /// Server sound identifiers.
    pub sound_ids: Vec<i32>,
}

impl Message for RemovedSounds {
    const COMMAND_ID: u16 = TOSERVER_REMOVED_SOUNDS;
}

impl EncodeMessage for RemovedSounds {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_removed_sounds(&self.sound_ids)
    }
}

/// Reports damage received by the local player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Damage {
    /// Damage amount.
    pub amount: u16,
}

impl Message for Damage {
    const COMMAND_ID: u16 = TOSERVER_DAMAGE;
}

impl EncodeMessage for Damage {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        Ok(encode_damage(self.amount))
    }
}

/// Requests the legacy respawn operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnLegacy;

impl Message for RespawnLegacy {
    const COMMAND_ID: u16 = TOSERVER_RESPAWN_LEGACY;
}

impl EncodeMessage for RespawnLegacy {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        Ok(encode_respawn_legacy())
    }
}

impl Message for ClientInfoUpdate {
    const COMMAND_ID: u16 = TOSERVER_UPDATE_CLIENT_INFO;
}

impl EncodeMessage for ClientInfoUpdate {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_update_client_info(self)
    }
}
