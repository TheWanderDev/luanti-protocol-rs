use crate::protocol::commands::{
    TOCLIENT_DETACHED_INVENTORY, TOCLIENT_FORMSPEC_PREPEND, TOCLIENT_INVENTORY_FORMSPEC,
    TOCLIENT_SHOW_FORMSPEC,
};
use crate::protocol::formspec::{
    DetachedInventoryUpdate, FormspecPrepend, InventoryFormspec, ShowFormspec,
    decode_detached_inventory, decode_formspec_prepend, decode_inventory_formspec,
    decode_show_formspec,
};
use crate::protocol::messages::{DecodeMessage, Message};

impl Message for InventoryFormspec {
    const COMMAND_ID: u16 = TOCLIENT_INVENTORY_FORMSPEC;
}

impl DecodeMessage for InventoryFormspec {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_inventory_formspec(payload)
    }
}

impl Message for DetachedInventoryUpdate {
    const COMMAND_ID: u16 = TOCLIENT_DETACHED_INVENTORY;
}

impl DecodeMessage for DetachedInventoryUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_detached_inventory(payload)
    }
}

impl Message for ShowFormspec {
    const COMMAND_ID: u16 = TOCLIENT_SHOW_FORMSPEC;
}

impl DecodeMessage for ShowFormspec {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_show_formspec(payload)
    }
}

impl Message for FormspecPrepend {
    const COMMAND_ID: u16 = TOCLIENT_FORMSPEC_PREPEND;
}

impl DecodeMessage for FormspecPrepend {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_formspec_prepend(payload)
    }
}
