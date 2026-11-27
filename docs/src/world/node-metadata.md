# Node Metadata

Node metadata payloads are decoded by [`crate::protocol::node_metadata`]. [`crate::protocol::node_metadata::NodeMetadata`] associates a node position with named fields and serialized inventory text. Each [`crate::protocol::node_metadata::NodeMetadataField`] contains a key, value, and privacy flag.

`decode_node_metadata_list(data, world_positions)` accepts metadata list versions 1 and 2; version 0 represents an empty list. The boolean selects whether positions are encoded as world coordinates or packed local block coordinates. The decoder rejects unsupported versions, duplicate positions, invalid local positions, invalid privacy flags, missing inventory terminators, malformed strings, and trailing bytes.

`decode_node_metadata_changed` reads a length-prefixed compressed payload, decompresses the zlib frame, verifies the consumed compressed length, and decodes the contained world-position list.

The metadata inventory is retained as serialized text. Parsing inventory contents is handled by [`crate::protocol::inventory`].
