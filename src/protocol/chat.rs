use bytes::Bytes;

use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum ChatMessageType {
    Raw = 0,
    Normal = 1,
    Announce = 2,
    System = 3,
}

impl TryFrom<u8> for ChatMessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_repr(value).ok_or(ProtocolError::InvalidChatMessageType(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub message_type: ChatMessageType,
    pub sender: String,
    pub message: String,
    pub timestamp: u64,
}

pub(crate) fn decode_chat_message(data: &[u8]) -> Result<ChatMessage, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version != 1 {
        return Err(ProtocolError::InvalidChatMessageVersion(version));
    }

    let message_type = ChatMessageType::try_from(reader.read_u8()?)?;
    let sender = reader.read_wide_string16()?;
    let message = reader.read_wide_string16()?;
    let timestamp = reader.read_u64()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidChatMessage);
    }

    Ok(ChatMessage {
        message_type,
        sender,
        message,
        timestamp,
    })
}

pub(crate) fn encode_chat_message(message: &str) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::ChatMessage));
    writer.write_wide_string16(message)?;

    Ok(writer.into_inner())
}
