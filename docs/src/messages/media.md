# Media

Media announcements, pushes, bunches, and client requests are handled by [`crate::protocol::media`].

The data models are [`crate::protocol::media::AnnouncedMediaFile`] (name and SHA-1 digest), [`crate::protocol::media::AnnounceMedia`] (file list and remote URL), [`crate::protocol::media::MediaFile`] (name and bytes), [`crate::protocol::media::MediaBunch`] (bunch count, index, and files), and [`crate::protocol::media::MediaPushUpdate`] (hash, filename, cache state, optional callback token, and optional file bytes).

Inbound decoders include `decode_media_push`, `decode_announce_media`, and `decode_media_bunch`. The bunch decoder accepts the negotiated protocol version because compression and payload layout vary by protocol generation. The compression helpers are in [`crate::protocol::compression`].

Outbound requests use `encode_request_media` and `encode_have_media`. The former serializes filenames; the latter serializes media tokens. This crate does not perform HTTP downloads, manage a disk cache, or decide when the client is ready to enter the world.
