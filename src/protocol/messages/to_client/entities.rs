use crate::protocol::active_object::{
    ActiveObjectMessage, ActiveObjectUpdate, decode_active_object_messages,
    decode_active_object_remove_add,
};
use crate::protocol::commands::{
    TOCLIENT_ACTIVE_OBJECT_MESSAGES, TOCLIENT_ACTIVE_OBJECT_REMOVE_ADD, TOCLIENT_ITEMDEF,
    TOCLIENT_NODEDEF, TOCLIENT_REMOVENODE,
};
use crate::protocol::itemdef::{DecodedItemDefinitions, ItemDefinition, decode_item_def};
use crate::protocol::mapblock::decode_remove_node;
use crate::protocol::messages::{DecodeMessage, DecodeMessageWithProtocolVersion, Message};
use crate::protocol::nodedef::{NodeDefinition, decode_node_def};
use glam::I16Vec3;

/// Server message containing active-object additions and removals.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveObjectRemoveAdd {
    /// Decoded object changes.
    pub update: ActiveObjectUpdate,
}

impl Message for ActiveObjectRemoveAdd {
    const COMMAND_ID: u16 = TOCLIENT_ACTIVE_OBJECT_REMOVE_ADD;
}

impl DecodeMessage for ActiveObjectRemoveAdd {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let update = decode_active_object_remove_add(payload)?;
        Ok(Self { update })
    }
}

/// Server message containing active-object updates.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveObjectMessagesUpdate {
    /// Decoded active-object messages.
    pub messages: Vec<ActiveObjectMessage>,
}

impl Message for ActiveObjectMessagesUpdate {
    const COMMAND_ID: u16 = TOCLIENT_ACTIVE_OBJECT_MESSAGES;
}

impl DecodeMessage for ActiveObjectMessagesUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let messages = decode_active_object_messages(payload)?;
        Ok(Self { messages })
    }
}

/// Server message containing a removed node position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveNodeUpdate {
    /// Position of the removed node.
    pub position: I16Vec3,
}

impl Message for RemoveNodeUpdate {
    const COMMAND_ID: u16 = TOCLIENT_REMOVENODE;
}

impl DecodeMessage for RemoveNodeUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let position = decode_remove_node(payload)?;
        Ok(Self { position })
    }
}

/// Server message containing the node definition registry.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeDefinitionsUpdate {
    /// Decoded node definitions.
    pub definitions: Vec<NodeDefinition>,
}

impl Message for NodeDefinitionsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_NODEDEF;
}

impl DecodeMessageWithProtocolVersion for NodeDefinitionsUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let definitions = decode_node_def(payload, protocol_version)?;
        Ok(Self { definitions })
    }
}

/// Server message containing item definitions and aliases.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemDefinitionsUpdate {
    /// Decoded item definitions.
    pub definitions: Vec<ItemDefinition>,
    /// Item-name aliases.
    pub aliases: Vec<(String, String)>,
}

impl Message for ItemDefinitionsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_ITEMDEF;
}

impl DecodeMessageWithProtocolVersion for ItemDefinitionsUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let (definitions, aliases): DecodedItemDefinitions =
            decode_item_def(payload, protocol_version)?;
        Ok(Self {
            definitions,
            aliases,
        })
    }
}
