use crate::protocol::commands::{TOSERVER_HAVE_MEDIA, TOSERVER_REQUEST_MEDIA};
use crate::protocol::media::{encode_have_media, encode_request_media};
use crate::protocol::messages::{EncodeMessage, Message};

/// Requests media files from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMedia {
    /// Requested media filenames.
    pub filenames: Vec<String>,
}

impl Message for RequestMedia {
    const COMMAND_ID: u16 = TOSERVER_REQUEST_MEDIA;
}

impl EncodeMessage for RequestMedia {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_request_media(&self.filenames)
    }
}

/// Acknowledges media files received by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HaveMedia {
    /// Media callback tokens acknowledged by the client.
    pub tokens: Vec<u32>,
}

impl Message for HaveMedia {
    const COMMAND_ID: u16 = TOSERVER_HAVE_MEDIA;
}

impl EncodeMessage for HaveMedia {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        encode_have_media(&self.tokens)
    }
}
