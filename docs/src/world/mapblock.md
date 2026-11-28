# MapBlock

Map and node payload codecs are in `protocol::mapblock`. A map block contains 16 × 16 × 16 nodes
(4096 total). Each `MapNode` stores a `u16` content ID and two parameter bytes. The low nibble of
`param1` is exposed as day light; the high nibble is night light.

`MapBlockData` contains block position, flags, lighting-complete value, decoded nodes, raw metadata
bytes, parsed node metadata, and the network-specific version byte.

For a complete server application message, use
`protocol::messages::to_client::BlockDataUpdate`, which implements
`DecodeMessageWithSerializationVersion`. It accepts the complete message and negotiated
serialization version. `protocol::mapblock::decode_map_block` remains a public standalone payload
codec for callers that already have the payload bytes.

Single-node updates are exposed as `to_client::AddNodeUpdate` and `to_client::RemoveNodeUpdate`.
Client block acknowledgements are exposed as `protocol::messages::to_server::GotBlocks` and
`DeletedBlocks`, which implement `EncodeMessage`.
