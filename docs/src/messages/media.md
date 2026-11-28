# Media

Media message types are exposed in `protocol::messages`.

## Server-to-client

`protocol::messages::to_client` exports `AnnounceMedia`, `MediaBunch`, and `MediaPushUpdate`.
These types implement `DecodeMessageWithProtocolVersion`; pass the negotiated protocol version.
The resulting models and media file types are defined in `protocol::media`.

## Client-to-server

`protocol::messages::to_server` exports `RequestMedia` and `HaveMedia`. Both implement
`EncodeMessage`.

This crate parses and serializes media messages. It does not download remote files, manage a disk
cache, or schedule media requests.
