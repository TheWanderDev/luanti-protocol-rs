use crate::protocol::chat::{ChatMessage, decode_chat_message};
use crate::protocol::commands::TOCLIENT_CHAT_MESSAGE;
use crate::protocol::messages::{DecodeMessage, EncodeMessage, Message};
use crate::protocol::writer::PacketWriter;

impl Message for ChatMessage {
    const COMMAND_ID: u16 = TOCLIENT_CHAT_MESSAGE;
}

impl DecodeMessage for ChatMessage {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_chat_message(payload)
    }
}

impl EncodeMessage for ChatMessage {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as Message>::COMMAND_ID);
        writer.write_u8(1);
        writer.write_u8(self.message_type as u8);
        writer.write_wide_string16(&self.sender)?;
        writer.write_wide_string16(&self.message)?;
        writer.write_u64(self.timestamp);
        Ok(writer.into_inner())
    }
}
