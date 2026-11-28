use crate::protocol::commands::TOCLIENT_INVENTORY;
use crate::protocol::inventory::{InventoryUpdate, decode_inventory};
use crate::protocol::messages::{DecodeMessageWithProtocolVersion, Message};

impl Message for InventoryUpdate {
    const COMMAND_ID: u16 = TOCLIENT_INVENTORY;
}

impl DecodeMessageWithProtocolVersion for InventoryUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_inventory(payload, protocol_version)
    }
}
