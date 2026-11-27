# MapBlock

Map and node payload codecs are in [`crate::protocol::mapblock`]. A map block contains 16 × 16 × 16 nodes (4096 total). Each [`crate::protocol::mapblock::MapNode`] stores a `u16` content ID and two parameter bytes. The low nibble of `param1` is exposed as day light; the high nibble is night light.

[`crate::protocol::mapblock::MapBlockData`] contains block position, flags, lighting-complete value, decoded nodes, raw metadata bytes, parsed node metadata, and the network-specific version byte.

`decode_map_block` accepts the payload and serialization version. The implementation selects node and compression layouts from that version. Incremental node updates are handled by `decode_add_node` and `decode_remove_node`.

The module also provides `encode_got_blocks` and `encode_deleted_blocks` for client block acknowledgements. These functions create game command payloads; they do not schedule block requests or manage the local map cache.
