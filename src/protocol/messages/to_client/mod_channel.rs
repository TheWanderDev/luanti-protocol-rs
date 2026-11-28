use crate::protocol::commands::{TOCLIENT_MODCHANNEL_MSG, TOCLIENT_MODCHANNEL_SIGNAL};
use crate::protocol::messages::{DecodeMessage, Message};
use crate::protocol::mod_channel::{
    ModChannelMessage, ModChannelSignalUpdate, decode_mod_channel_message,
    decode_mod_channel_signal,
};

impl Message for ModChannelMessage {
    const COMMAND_ID: u16 = TOCLIENT_MODCHANNEL_MSG;
}

impl DecodeMessage for ModChannelMessage {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_mod_channel_message(payload)
    }
}

/// Server notification describing a mod-channel state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelSignal {
    /// Decoded update, or `None` when the server sent an unknown signal value.
    pub update: Option<ModChannelSignalUpdate>,
}

impl Message for ModChannelSignal {
    const COMMAND_ID: u16 = TOCLIENT_MODCHANNEL_SIGNAL;
}

impl DecodeMessage for ModChannelSignal {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let update = decode_mod_channel_signal(payload)?;
        Ok(Self { update })
    }
}
