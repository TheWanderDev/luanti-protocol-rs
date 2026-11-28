use crate::protocol::commands::TOCLIENT_NODEMETA_CHANGED;
use crate::protocol::messages::{DecodeMessage, Message};
use crate::protocol::node_metadata::{NodeMetadata, decode_node_metadata_changed};

/// Node metadata entries sent by the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMetadataUpdate {
    /// Updated node metadata entries.
    pub entries: Vec<NodeMetadata>,
}

impl Message for NodeMetadataUpdate {
    const COMMAND_ID: u16 = TOCLIENT_NODEMETA_CHANGED;
}

impl DecodeMessage for NodeMetadataUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let entries = decode_node_metadata_changed(payload)?;
        Ok(Self { entries })
    }
}
