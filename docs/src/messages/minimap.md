# Minimap Modes

Minimap mode updates are decoded by [`crate::protocol::minimap::decode_minimap_modes`]. The returned [`crate::protocol::minimap::MinimapModesUpdate`] contains the available modes and selected mode index.

Each [`crate::protocol::minimap::MinimapMode`] contains a numeric mode type, label, size, texture name, and scale. The decoder reads the mode count and selected mode, validates that the packet has enough bytes for the declared minimum per-entry size, parses each entry, and rejects trailing bytes.

The module only decodes the mode list. It does not load textures, select a visual mode, or render a minimap.
