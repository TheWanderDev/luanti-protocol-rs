# Architecture

The crate is a codec library. Public modules are exposed through [`crate::protocol`], while [`crate::ProtocolError`] is re-exported at the crate root.

## Data flow

A caller receives a UDP datagram from its transport layer and passes its bytes to [`crate::protocol::packet::decode_datagram`]. The returned [`crate::protocol::packet::DecodedDatagram`] describes the peer ID, channel, optional reliable sequence number, and decoded MTP body. The transport layer remains responsible for sequence tracking, ACK scheduling, and split-message assembly.

After transport processing, a game payload begins with a big-endian `u16` command ID. Command enums live in [`crate::protocol::commands`]. Payload-specific modules decode the remaining bytes into typed structures. Outbound helpers write the command ID and payload into `Bytes`.

## Module boundaries

- `packet`: MTP framing only; it does not manage a connection.
- `reader` and `writer`: primitive field codecs shared by payload modules.
- `compression`: zlib and Zstandard frame decompression helpers.
- `commands`: command IDs and client command channel/reliability metadata.
- `messages`, `player`, `chat`, `client_commands`, `inventory_action`, `media`, `mod_channel`: selected outbound encoders and inbound decoders.
- `mapblock`, `nodedef`, `tiledef`, `node_metadata`, `itemdef`, `inventory`, `active_object`: structured world and entity payloads.
- `status`, `server_state`, `hud`, `particle`, `formspec`, `minimap`: server-to-client state updates.

## Ownership

[`crate::protocol::reader::PacketReader`] borrows its input slice. A successful `read_bytes` returns a slice with the same input lifetime; it does not allocate a new byte buffer. Decoded models that retain text or binary values generally own those values. [`crate::protocol::writer::PacketWriter`] accumulates output in `BytesMut` and freezes it into `Bytes`.

## API reference

Open the [Rust API Reference](../api/luanti_protocol_rs/index.html) for every public item and its exact signature.
