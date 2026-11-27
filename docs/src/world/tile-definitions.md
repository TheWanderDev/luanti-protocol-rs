# Tile Definitions

Tile, node-box, and content-feature wire models are in [`crate::protocol::tiledef`]. [`crate::protocol::tiledef::ContentFeaturesTiles`] groups node groups, parameter types, draw type, mesh and scale data, alpha/color/palette settings, liquid behavior, node boxes, tile arrays, overlays, special tiles, lighting and interaction flags, sounds, and legacy orientation flags.

[`crate::protocol::tiledef::NodeBoxDefinition`] contains a box kind and its [`crate::protocol::tiledef::NodeBoxPart`] list. Each part has minimum/maximum coordinates and a group value. [`crate::protocol::tiledef::TileDef`] stores a texture name, animation description, culling and tiling flags, plus optional color, scale, and alignment fields. `TileAnimation` models the supported animation variants; `SimpleSoundSpec` stores a sound name and its fixed parameter bytes.

`decode_content_features_tiles` reads the tile-related portion of content features. `encode_content_features_tiles` writes it. These functions are useful when inspecting node definitions, but they do not load textures, create meshes, or apply rendering state.
