# Public API and Compatibility Notes

## Public entry points

The crate exposes its API through `luanti_protocol_rs::protocol` and re-exports
`luanti_protocol_rs::ProtocolError` at the crate root. The directional application-message
modules are also re-exported at the crate root for direct discovery.

| Purpose | Public entry point |
|---|---|
| Client-to-server messages | `luanti_protocol_rs::to_server` |
| Server-to-client messages | `luanti_protocol_rs::to_client` |
| Complete application messages | `protocol::messages` |
| MTP datagram framing | `protocol::packet` |
| Primitive parsing and writing | `protocol::reader::PacketReader`, `protocol::writer::PacketWriter` |
| Protocol-version-aware decoding | `DecodeMessageWithProtocolVersion` |
| Serialization-version-aware decoding | `DecodeMessageWithSerializationVersion` |
| Standalone payload codecs | Public functions in the relevant `protocol` submodules |

The original paths `protocol::messages::to_server` and `protocol::messages::to_client` remain
available and refer to the same modules as the crate-root re-exports.

## Message traits

`Message` defines the command ID and validates the command prefix. `EncodeMessage` serializes a
complete application message. `DecodeMessage` parses a complete message without an explicit version
argument. `DecodeMessageWithProtocolVersion` and
`DecodeMessageWithSerializationVersion` parse messages whose payload layout depends on the
negotiated session version.

All message decoders receive the command ID as part of their input. Do not strip the command ID
before calling a typed decoder, and do not pass an MTP datagram to an application-message decoder.

## Crate-private payload helpers

Some low-level payload functions are `pub(crate)` because the public typed message wrappers
already provide the supported network API. Examples include the chat encoder/decoder,
inventory-action encoder, and minimap-mode decoder. Downstream crates should use
`to_server::ChatMessage`, `to_server::InventoryAction`, and `to_client::MinimapModesUpdate`
respectively.

Standalone operations remain public where they form independent APIs, including packet framing,
primitive reader/writer types, compression helpers, map-block decoding, node-metadata list
processing, formspec parsing, node transforms, and tile-definition codecs. Consult generated
rustdoc for exact visibility and signatures.

## Compatibility

The current source advertises protocol 37–55, serialization version 29, and formspec API version
12. Protocol 55 adds `move_camera` to `GenericActiveObjectAttachment`; downstream struct literals
must provide the new field. The protocol crate package version is `0.2.0`.
