# HUD

HUD payload decoders are implemented in [`crate::protocol::hud`]. The module models additions, removals, and changes to HUD elements.

[`crate::protocol::hud::HudAdd`] contains the element ID and type, position, name, scale, text and numeric fields, alignment, offsets, world position, size, z-index, secondary text, style, and hideability. `HudSize` represents the size format used by the decoder.

`decode_hud_add(data, protocol_version)` and `decode_hud_change(data, protocol_version)` receive the negotiated protocol version because the encoded layout changes by version. `decode_hud_remove` reads the element ID. Changes are represented by [`crate::protocol::hud::HudChange`], [`crate::protocol::hud::HudChangeStat`], and [`crate::protocol::hud::HudChangeValue`].

The decoder only parses update data. It does not create, render, or mutate HUD elements.
