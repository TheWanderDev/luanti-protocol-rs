use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ProtocolError {
    #[error("packet is truncated")]
    Truncated,
    #[error("packet exceeds the maximum MTP packet size")]
    PacketTooLarge,
    #[error("invalid MTP channel: {0}")]
    InvalidChannel(u8),
    #[error("invalid protocol identifier: {0:#010x}")]
    InvalidProtocolId(u32),
    #[error("unsupported MTP packet type: {0}")]
    UnsupportedPacketType(u8),
    #[error("invalid MTP control type: {0}")]
    InvalidControlType(u8),
    #[error("unexpected packet command: {0:#06x}")]
    UnexpectedCommand(u16),
    #[error("invalid string length")]
    InvalidStringLength,
    #[error("invalid hello packet")]
    InvalidHello,
    #[error("invalid negotiated protocol version: {0}")]
    InvalidProtocolVersion(u16),
    #[error("invalid serialization version: {0}")]
    InvalidSerializationVersion(u8),
    #[error("invalid authentication packet")]
    InvalidAuthPacket,
    #[error("invalid split packet")]
    InvalidSplitPacket,
    #[error("invalid reliable packet")]
    InvalidReliablePacket,
    #[error("invalid wide string encoding")]
    InvalidWideString,
    #[error("unsupported chat message version: {0}")]
    InvalidChatMessageVersion(u8),
    #[error("invalid chat message type: {0}")]
    InvalidChatMessageType(u8),
    #[error("invalid chat message packet")]
    InvalidChatMessage,
    #[error("invalid player packet")]
    InvalidPlayerPacket,
    #[error("invalid player value: {0}")]
    InvalidPlayerValue(&'static str),
    #[error("invalid sound packet: {0}")]
    InvalidSound(&'static str),
    #[error("invalid map block: {0}")]
    InvalidMapBlock(&'static str),
    #[error("invalid block acknowledgment batch size")]
    InvalidBlockBatch,
    #[error("invalid node definition: {0}")]
    InvalidNodeDef(&'static str),
    #[error("invalid node definition for content id {content_id} ({name}): {reason}")]
    InvalidNodeDefContext {
        content_id: u16,
        name: String,
        reason: String,
    },
    #[error("invalid node definition: unsupported node box type {0}")]
    UnsupportedNodeBoxType(u8),
    #[error("invalid node definition: unsupported node box version {0}")]
    UnsupportedNodeBoxVersion(u8),
    #[error("invalid item definition: {0}")]
    InvalidItemDef(&'static str),
    #[error("invalid inventory: {0}")]
    InvalidInventory(&'static str),
    #[error("invalid media packet: {0}")]
    InvalidMedia(&'static str),
    #[error("invalid HUD packet: {0}")]
    InvalidHud(&'static str),
    #[error("invalid active object packet: {0}")]
    InvalidActiveObject(&'static str),
    #[error("invalid particle packet: {0}")]
    InvalidParticle(&'static str),
    #[error("invalid mod channel packet: {0}")]
    InvalidModChannel(&'static str),
    #[error("invalid minimap packet: {0}")]
    InvalidMinimap(&'static str),
    #[error("invalid formspec: {0}")]
    InvalidFormspec(&'static str),
}
