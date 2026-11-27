# Node Definitions

Node-definition registry data and codecs are in [`crate::protocol::nodedef`]. The module models content features indexed by numeric content IDs and maintains a registry for name/ID lookup.

The public data types include [`crate::protocol::nodedef::NodeDefinition`], [`crate::protocol::nodedef::NodeLightingFlags`], and [`crate::protocol::nodedef::NodeRegistry`]. `CONTENT_IGNORE` is the declared ignore content ID. `decode_node_def` parses the server definition payload into the registry representation; `encode_node_def` serializes definitions in the opposite direction.

Node definitions refer to tile and node-box data represented in [`crate::protocol::tiledef`]. The decoder validates unsupported node-box types/versions and reports contextual errors that can include the content ID and node name.

Use the negotiated protocol version when the caller's dispatch path requires version-specific layouts. Do not treat a node name as a stable numeric content ID across sessions; IDs are supplied by the server's content registry.
