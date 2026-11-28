//! Typed application messages grouped by their network direction.
//!
//! Message codecs operate on complete application messages, including the two-byte command ID.
//! MTP framing belongs to [`super::packet`].
//!
//! # Chat and mod channels
//!
//! ```no_run
//! use luanti_protocol_rs::protocol::messages::{EncodeMessage, to_server};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let chat = to_server::ChatMessage {
//!     message: String::from("Hello"),
//! };
//! let chat_bytes = chat.encode()?;
//!
//! let join = to_server::ModChannelJoin {
//!     channel: String::from("example:updates"),
//! };
//! let join_bytes = join.encode()?;
//! # let _ = (chat_bytes, join_bytes);
//! # Ok(())
//! # }
//! ```

//! # Decoding server messages
//!
//! ```no_run
//! use luanti_protocol_rs::ProtocolError;
//! use luanti_protocol_rs::protocol::messages::{DecodeMessage, to_client};
//!
//! fn decode_hello(data: &[u8]) -> Result<to_client::Hello, ProtocolError> {
//!     to_client::Hello::decode(data)
//! }
//! ```
//!
//! # Decoding version-dependent messages
//!
//! ```no_run
//! use luanti_protocol_rs::ProtocolError;
//! use luanti_protocol_rs::protocol::messages::{DecodeMessageWithProtocolVersion, to_client};
//!
//! fn decode_inventory(
//!     data: &[u8],
//!     protocol_version: u16,
//! ) -> Result<to_client::InventoryUpdate, ProtocolError> {
//!     to_client::InventoryUpdate::decode(data, protocol_version)
//! }
//! ```
//!
//! # Client handshake
//!
//! ```no_run
//! use luanti_protocol_rs::protocol::messages::{EncodeMessage, to_server};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let init = to_server::Init {
//!     player_name: String::from("player"),
//!     protocol_min: 37,
//!     protocol_max: 55,
//!     serialization_version: 29,
//! };
//! let application_message = init.encode()?;
//! # let _ = application_message;
//! # Ok(())
//! # }
//! ```

use bytes::Bytes;

use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;

pub mod to_client;
pub mod to_server;

/// Reads the command identifier at the start of a complete application message.
///
/// # Errors
///
/// Returns [`ProtocolError::Truncated`] when the input does not contain a command identifier.
pub fn command_id(data: &[u8]) -> Result<u16, ProtocolError> {
    PacketReader::new(data).read_u16()
}

/// Identifies an application message by its command ID.
pub trait Message {
    /// Command ID encoded at the beginning of the application message.
    const COMMAND_ID: u16;

    /// Validates the command ID and returns the application payload.
    ///
    /// The input must contain the complete application message. MTP packet framing is not
    /// accepted here.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError`] when the message is truncated or has a different command ID.
    fn decode_payload(data: &[u8]) -> Result<&[u8], ProtocolError> {
        let mut reader = PacketReader::new(data);
        let command_id = reader.read_u16()?;
        if command_id != Self::COMMAND_ID {
            return Err(ProtocolError::UnexpectedCommand(command_id));
        }

        data.get(2..).ok_or(ProtocolError::Truncated)
    }
}

/// Encodes a complete application message, including its command ID.
pub trait EncodeMessage: Message {
    /// Serializes this message into its wire representation.
    fn encode(&self) -> Result<Bytes, ProtocolError>;
}

/// Decodes a complete application message and validates its command ID.
pub trait DecodeMessage: Message + Sized {
    /// Parses a complete application message.
    fn decode(data: &[u8]) -> Result<Self, ProtocolError>;
}

/// Decodes an application message whose payload depends on the negotiated protocol version.
///
/// Implement this trait instead of [`DecodeMessage`] when the wire representation changes
/// between protocol versions. The input includes the two-byte command ID.
pub trait DecodeMessageWithProtocolVersion: Message + Sized {
    /// Parses a complete application message using the negotiated protocol version.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError`] when the command ID, payload, or protocol-specific fields
    /// are invalid.
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, ProtocolError>;
}

/// Decodes an application message whose payload depends on the negotiated serialization version.
///
/// The input includes the two-byte command ID.
pub trait DecodeMessageWithSerializationVersion: Message + Sized {
    /// Parses a complete application message using the negotiated serialization version.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError`] when the command ID, payload, or serialization-specific fields
    /// are invalid.
    fn decode(data: &[u8], serialization_version: u8) -> Result<Self, ProtocolError>;
}
