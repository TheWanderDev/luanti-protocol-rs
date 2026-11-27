# Protocol Codec Documentation

This book describes the public API and wire-format behavior implemented by this crate. It is organized around the source modules exposed from `protocol` and the crate-level `ProtocolError` type.

The crate provides binary packet framing, payload encoders and decoders, data models for server updates, and compression helpers. It does not implement a UDP socket, peer lifecycle, retransmission scheduler, reliable-window manager, split-packet reassembly queue, or a complete login client. Those responsibilities belong to the networking layer that consumes these codecs.

## Source layout

| Area | Source modules | Responsibility |
|---|---|---|
| Transport framing | `packet`, `constant` | MTP datagram header and packet body codecs |
| Primitive codecs | `reader`, `writer`, `vectors`, `compression` | Binary fields, vectors, compressed payloads |
| Commands and handshake | `commands`, `messages`, `client_commands` | Command IDs and selected client/server handshake payloads |
| Gameplay messages | `chat`, `player`, `status`, `server_state`, `hud`, `particle` | Player input and server state updates |
| Content and world | `mapblock`, `nodedef`, `tiledef`, `node_metadata`, `node_transform`, `itemdef` | Map and content definitions |
| Session features | `inventory`, `inventory_action`, `media`, `formspec`, `mod_channel`, `minimap`, `active_object` | Inventory, media, forms, channels, minimap, entities |

## Build both documentation sets

`cargo doc --no-deps` creates the Rust API reference. `mdbook build docs` creates this guide. The `rustdoc-links` preprocessor resolves Rust item references written in Rustdoc-style Markdown syntax, for example [`crate::protocol::reader::PacketReader`].

Open the [Rust API Reference](api/luanti_protocol_rs/index.html). For a combined static site, the build script copies `target/doc/luanti_protocol_rs/` into `docs/src/api/luanti_protocol_rs/` before mdBook renders the guide.

## Compatibility scope

The crate constants currently declare protocol versions 37 through 54, serialization version 29, client version 5.18.0, and formspec API version 12. These are the values in this source snapshot; see [Protocol Versions](reference/protocol-versions.md) and [Upstream Compatibility](reference/upstream-compatibility.md) for the distinction between crate declarations and independently verified upstream behavior.
