use crate::protocol::commands::{TOSERVER_INVENTORY_FIELDS, TOSERVER_NODEMETA_FIELDS};
use crate::protocol::formspec::{
    InventoryFields, NodeMetadataFields, decode_inventory_fields_payload,
    decode_node_metadata_fields, encode_inventory_fields, encode_node_metadata_fields,
};
use crate::protocol::messages::{DecodeMessage, EncodeMessage, Message};

impl Message for InventoryFields {
    const COMMAND_ID: u16 = TOSERVER_INVENTORY_FIELDS;
}

impl EncodeMessage for InventoryFields {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_inventory_fields(&self.form_name, &self.fields)
    }
}

impl DecodeMessage for InventoryFields {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_inventory_fields_payload(payload)
    }
}

impl Message for NodeMetadataFields {
    const COMMAND_ID: u16 = TOSERVER_NODEMETA_FIELDS;
}

impl EncodeMessage for NodeMetadataFields {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_node_metadata_fields(self.position, &self.form_name, &self.fields)
    }
}

impl DecodeMessage for NodeMetadataFields {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_node_metadata_fields(payload)
    }
}
