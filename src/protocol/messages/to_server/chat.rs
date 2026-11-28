use crate::protocol::chat::encode_chat_message;
use crate::protocol::commands::TOSERVER_CHAT_MESSAGE;
use crate::protocol::messages::{DecodeMessage, EncodeMessage, Message};
use crate::protocol::reader::PacketReader;

/// Chat text sent by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub message: String,
}

impl Message for ChatMessage {
    const COMMAND_ID: u16 = TOSERVER_CHAT_MESSAGE;
}

impl EncodeMessage for ChatMessage {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_chat_message(&self.message)
    }
}

impl DecodeMessage for ChatMessage {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let message = reader.read_wide_string16()?;
        if !reader.is_empty() {
            return Err(crate::errors::ProtocolError::InvalidChatMessage);
        }
        Ok(Self { message })
    }
}
