use crate::protocol::commands::{
    TOSERVER_MODCHANNEL_JOIN, TOSERVER_MODCHANNEL_LEAVE, TOSERVER_MODCHANNEL_MSG,
};
use crate::protocol::messages::{EncodeMessage, Message};
use crate::protocol::mod_channel::{
    encode_mod_channel_join, encode_mod_channel_leave, encode_mod_channel_message,
};

/// Requests membership in a mod channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelJoin {
    pub channel: String,
}

impl Message for ModChannelJoin {
    const COMMAND_ID: u16 = TOSERVER_MODCHANNEL_JOIN;
}

impl EncodeMessage for ModChannelJoin {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_mod_channel_join(&self.channel)
    }
}

/// Leaves a mod channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelLeave {
    pub channel: String,
}

impl Message for ModChannelLeave {
    const COMMAND_ID: u16 = TOSERVER_MODCHANNEL_LEAVE;
}

impl EncodeMessage for ModChannelLeave {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_mod_channel_leave(&self.channel)
    }
}

/// Sends text to a mod channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelMessage {
    pub channel: String,
    pub message: String,
}

impl Message for ModChannelMessage {
    const COMMAND_ID: u16 = TOSERVER_MODCHANNEL_MSG;
}

impl EncodeMessage for ModChannelMessage {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_mod_channel_message(&self.channel, &self.message)
    }
}
