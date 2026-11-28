use crate::protocol::commands::{TOSERVER_INTERACT, TOSERVER_PLAYERITEM, TOSERVER_PLAYERPOS};
use crate::protocol::messages::{EncodeMessage, Message};
use crate::protocol::player::{
    InteractAction, PlayerPositionUpdate, PointedThing, encode_interact, encode_player_item,
    encode_player_position,
};

impl Message for PlayerPositionUpdate {
    const COMMAND_ID: u16 = TOSERVER_PLAYERPOS;
}

impl EncodeMessage for PlayerPositionUpdate {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_player_position(self)
    }
}

/// Changes the selected hotbar item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerItem {
    /// Selected hotbar index.
    pub index: u16,
}

impl Message for PlayerItem {
    const COMMAND_ID: u16 = TOSERVER_PLAYERITEM;
}

impl EncodeMessage for PlayerItem {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        Ok(encode_player_item(self.index))
    }
}

/// Sends an interaction together with the current player movement state.
#[derive(Debug, Clone, PartialEq)]
pub struct Interact {
    /// Interaction operation.
    pub action: InteractAction,
    /// Selected hotbar index.
    pub wield_index: u16,
    /// Target of the interaction.
    pub pointed: PointedThing,
    /// Player movement state accompanying the interaction.
    pub player: PlayerPositionUpdate,
}

impl Message for Interact {
    const COMMAND_ID: u16 = TOSERVER_INTERACT;
}

impl EncodeMessage for Interact {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_interact(self.action, self.wield_index, self.pointed, &self.player)
    }
}
