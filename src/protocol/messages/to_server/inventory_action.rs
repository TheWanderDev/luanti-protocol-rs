use crate::protocol::commands::TOSERVER_INVENTORY_ACTION;
use crate::protocol::inventory_action::{InventoryAction, encode_inventory_action};
use crate::protocol::messages::{EncodeMessage, Message};

impl Message for InventoryAction {
    const COMMAND_ID: u16 = TOSERVER_INVENTORY_ACTION;
}

impl EncodeMessage for InventoryAction {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_inventory_action(self)
    }
}
