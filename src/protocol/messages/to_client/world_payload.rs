use glam::I16Vec3;

use crate::errors::ProtocolError;
use crate::protocol::commands::{TOCLIENT_ADDNODE, TOCLIENT_BLOCKDATA};
use crate::protocol::mapblock::{MapBlockData, MapNode, decode_add_node, decode_map_block};
use crate::protocol::messages::{DecodeMessageWithSerializationVersion, Message};

/// Decoded map-block update received from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDataUpdate {
    /// Position and node data of the map block.
    pub block: MapBlockData,
}

impl Message for BlockDataUpdate {
    const COMMAND_ID: u16 = TOCLIENT_BLOCKDATA;
}

impl DecodeMessageWithSerializationVersion for BlockDataUpdate {
    fn decode(data: &[u8], serialization_version: u8) -> Result<Self, ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let block = decode_map_block(payload, serialization_version)?;
        Ok(Self { block })
    }
}

/// Decoded single-node update received from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddNodeUpdate {
    /// Position of the changed node.
    pub position: I16Vec3,
    /// New node value.
    pub node: MapNode,
    /// Whether existing node metadata is retained.
    pub keep_metadata: bool,
}

impl Message for AddNodeUpdate {
    const COMMAND_ID: u16 = TOCLIENT_ADDNODE;
}

impl DecodeMessageWithSerializationVersion for AddNodeUpdate {
    fn decode(data: &[u8], serialization_version: u8) -> Result<Self, ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let (position, node, keep_metadata) = decode_add_node(payload, serialization_version)?;
        Ok(Self {
            position,
            node,
            keep_metadata,
        })
    }
}
