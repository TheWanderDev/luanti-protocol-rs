# HUD

HUD network updates are exposed as typed server-to-client messages in
`protocol::messages::to_client`.

The public exports include `HudAdd`, `HudRemove`, `HudChange`, `HudFlagsUpdate`, and
`HudParamUpdate`. Use the decode trait implemented by each selected type. `HudChange` and
`HudAdd` are models from `protocol::hud`; other state models are exported from
`protocol::server_state`.

The lower-level HUD payload decoders are internal to the message wrappers. This crate parses the
wire data; it does not create, render, or mutate HUD elements.
