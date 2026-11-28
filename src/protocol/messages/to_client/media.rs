use crate::protocol::commands::{TOCLIENT_ANNOUNCE_MEDIA, TOCLIENT_MEDIA, TOCLIENT_MEDIA_PUSH};
use crate::protocol::media::{
    AnnounceMedia, MediaBunch, MediaPushUpdate, decode_announce_media, decode_media_bunch,
    decode_media_push,
};
use crate::protocol::messages::{DecodeMessageWithProtocolVersion, Message};

impl Message for AnnounceMedia {
    const COMMAND_ID: u16 = TOCLIENT_ANNOUNCE_MEDIA;
}

impl DecodeMessageWithProtocolVersion for AnnounceMedia {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_announce_media(payload, protocol_version)
    }
}

impl Message for MediaBunch {
    const COMMAND_ID: u16 = TOCLIENT_MEDIA;
}

impl DecodeMessageWithProtocolVersion for MediaBunch {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_media_bunch(payload, protocol_version)
    }
}

impl Message for MediaPushUpdate {
    const COMMAND_ID: u16 = TOCLIENT_MEDIA_PUSH;
}

impl DecodeMessageWithProtocolVersion for MediaPushUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_media_push(payload, protocol_version)
    }
}
