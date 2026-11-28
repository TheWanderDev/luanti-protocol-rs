#  API Guide

This page is a compact, retrieval-friendly map of the crate. It states the supported entry points,
input format, version requirements, and subsystem boundaries without requiring an AI agent to infer
behavior from names alone.

## Crate identity

- Crate: `luanti-protocol-rs`
- Language edition: Rust 2024
- Role: binary protocol codec library
- Error type: `luanti_protocol_rs::ProtocolError`
- Declared protocol range: 37–55
- Serialization version: 29
- Forms API version: 12
- Client version metadata: 5.19.0-dev

## Choose the correct layer

| Input or task | Public entry point | Output or result |
|---|---|---|
| One MTP datagram | `protocol::packet::decode_datagram` | `DecodedDatagram` |
| Primitive binary fields | `protocol::reader::PacketReader` | Borrowed fields or `ProtocolError` |
| Build primitive payload bytes | `protocol::writer::PacketWriter` | `Bytes` |
| Read an application command ID | `protocol::messages::command_id` | `u16` |
| Decode a complete server message | `protocol::messages::to_client` plus a decode trait | Typed server message |
| Encode a complete client message | `protocol::messages::to_server` plus `EncodeMessage` | `Bytes` with command ID |
| Decode a standalone map block | `protocol::mapblock::decode_map_block` | `MapBlockData` |
| Decode a standalone node-metadata list | `protocol::node_metadata::decode_node_metadata_list` | `Vec<NodeMetadata>` |
| Decode or encode content-feature tile data | `protocol::tiledef` | `ContentFeaturesTiles` |
| Decode compressed content | `protocol::compression` | Decoded byte vector |

## Message traits

All message input and output is application-layer data, not an MTP datagram.

- `Message` associates a type with its `COMMAND_ID` and validates the command prefix through
  `decode_payload`.
- `EncodeMessage` encodes the command ID followed by the message payload.
- `DecodeMessage` decodes a complete message whose layout does not require an explicit version.
- `DecodeMessageWithProtocolVersion` decodes a complete message using the negotiated protocol
  version.
- `DecodeMessageWithSerializationVersion` decodes a complete message using the negotiated
  serialization version.

### Decode example

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{DecodeMessage, to_client};

fn decode_hello(data: &[u8]) -> Result<to_client::Hello, ProtocolError> {
    to_client::Hello::decode(data)
}
```

### Encode example

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{EncodeMessage, to_server};

fn main() -> Result<(), ProtocolError> {
    let encoded = to_server::ChatMessage {
        message: String::from("Hello"),
    }
    .encode()?;
    let _wire_bytes: &[u8] = encoded.as_ref();
    Ok(())
}
```

## Public message families

### Server to client

Types are exported from `protocol::messages::to_client`.

| Family | Public message types |
|---|---|
| Handshake/authentication | `Hello`, `AuthAccept`, `AccessDenied`, `AccessDeniedLegacy`, `AcceptSudoMode`, `DenySudoMode`, `SrpChallenge` |
| Chat | `ChatMessage` |
| World | `BlockDataUpdate`, `AddNodeUpdate`, `RemoveNodeUpdate`, `NodeDefinitionsUpdate`, `ItemDefinitionsUpdate`, `NodeMetadataUpdate` |
| Inventory | `InventoryUpdate` |
| Media | `AnnounceMedia`, `MediaBunch`, `MediaPushUpdate` |
| Player/status | `ServerPlayerMovement`, `LegacyDeathscreen`, `MovementParameters`, `TimeOfDayUpdate`, `HpUpdate`, `PlayerSpeedUpdate`, `PlayerBreathUpdate`, `LocalPlayerAnimationsUpdate` |
| HUD/state | `HudAdd`, `HudRemove`, `HudChange`, `HudFlagsUpdate`, `HudParamUpdate`, `CameraModeUpdate`, `FovUpdate` |
| Environment/audio | `LightingUpdate`, `SkyUpdate`, `DayNightRatioUpdate`, `EyeOffsetUpdate`, `CloudParamsUpdate`, `SunUpdate`, `MoonUpdate`, `StarsUpdate`, `PlaySoundUpdate`, `StopSoundUpdate`, `FadeSoundUpdate` |
| Entities/particles | `ActiveObjectRemoveAdd`, `ActiveObjectMessagesUpdate`, `ParticleParameters`, `ParticleSpawnerParameters`, `ParticleBatchUpdate`, `DeleteParticleSpawner` |
| Other state | `PrivilegesUpdate`, `PlayerListUpdate`, `RelativePlayerMove`, `CsmRestrictionFlagsUpdate`, `MinimapModesUpdate`, `ModChannelMessage`, `ModChannelSignal`, `InventoryFormspec`, `ShowFormspec`, `DetachedInventoryUpdate`, `FormspecPrepend` |

Use the decode trait implemented by the selected type. Some types require the negotiated protocol
or serialization version; inspect the trait implementation and pass that exact session value.

### Client to server

Types are exported from `protocol::messages::to_server`.

| Family | Public message types |
|---|---|
| Handshake/authentication | `Init`, `Init2`, `FirstSrp`, `SrpBytesA`, `SrpBytesM`, `ClientReady` |
| Player | `PlayerPositionUpdate`, `PlayerItem`, `Interact` |
| Inventory/forms | `InventoryAction`, `InventoryFields`, `NodeMetadataFields` |
| Chat/channels | `ChatMessage`, `ModChannelJoin`, `ModChannelLeave`, `ModChannelMessage` |
| Media | `RequestMedia`, `HaveMedia` |
| Other commands | `GotBlocks`, `DeletedBlocks`, `RemovedSounds`, `Damage`, `RespawnLegacy`, `ClientInfoUpdate` |

Use `EncodeMessage::encode` to produce the complete application message. The transport layer must
add MTP framing separately.

## Ownership and error behavior

- `PacketReader<'a>` borrows the input slice. `read_bytes`, `read_string16`, and `read_string32`
  return borrowed byte slices.
- Wide-string decoding and domain models that retain values allocate owned data.
- `PacketWriter` accumulates bytes in `BytesMut` and returns immutable `Bytes` from `into_inner`.
- Packet and message decoders return `ProtocolError` for truncated, malformed, unsupported, or
  inconsistent input.
- Compression helpers return static-string errors; higher-level payload decoders map failures to
  `ProtocolError` where appropriate.
- Do not continue parsing after a decoding error.

## Public API versus implementation details

The intended public entry points are the public message types and traits, public standalone codecs, and public models documented in the generated rustdoc. Several lower-level payload functions remain
`pub(crate)` because their behavior is already available through typed message wrappers. They are
internal dispatch details, not missing public API. Do not expose them as separate APIs unless a
concrete external use case requires a supported standalone payload contract.

## Protocol 55 attachment flags

Protocol 55 extends the flags byte in the generic active-object `ATTACH_TO` payload:

- `0x01`: force visibility on attach (`visible_on_attach`).
- `0x02`: move the player camera with the attachment (`move_camera`).

For protocol 37–54, the sender writes only the visibility bit. The decoder reads the same byte
position for both layouts and extracts each bit independently, preserving older behavior.

See [Protocol Versions](reference/protocol-versions.md) and
[Upstream Compatibility](reference/upstream-compatibility.md) for the verified wire behavior.

## Integration checklist

1. Decode MTP datagrams before dispatching application messages.
2. Reassemble split transport messages in the transport layer before application decoding.
3. Read the two-byte application command ID and dispatch to the matching public message type.
4. Pass the complete application message, including its command ID, to a message decoder.
5. Pass the negotiated protocol or serialization version to version-aware decoders.
6. Handle `ProtocolError` at the packet boundary; do not silently replace malformed fields.
7. Keep sockets, ACKs, retransmission, peer state, and rendering outside this crate.
