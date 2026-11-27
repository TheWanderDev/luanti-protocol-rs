# Server State Updates

Environment and status updates are decoded by [`crate::protocol::server_state`] and [`crate::protocol::status`]. These modules return typed values; applying them to a renderer or game state is the caller's responsibility.

## Environment and audiovisual state

`server_state` includes lighting and auto-exposure data, sky colors and parameters, cloud settings, sun/moon/stars, day-night ratio, eye offsets, local player animation ranges, and sound play/stop/fade updates. It also decodes privileges, player-list operations, FOV, camera mode, CSM restriction flags, HUD flags/parameters, relative player movement, and particle-spawner deletion.

## Player status

`status` decodes movement parameters, time of day, HP, breath, and added player velocity. Its data models include [`crate::protocol::status::MovementParameters`], [`crate::protocol::status::TimeOfDayUpdate`], [`crate::protocol::status::HpUpdate`], and [`crate::protocol::status::PlayerSpeedUpdate`].

Some server-state decoders accept a protocol version because optional fields depend on the negotiated wire format. Preserve that version at the dispatch boundary.
