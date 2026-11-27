# Player State and Interaction

Player-related codecs are in [`crate::protocol::player`]. The module includes inbound movement/death-screen decoders and outbound player input encoders.

## Server-to-client data

`decode_move_player` returns [`crate::protocol::player::ServerPlayerMovement`], containing position, pitch, and yaw. The decoder converts the wire angles to radians and applies the pitch sign conversion present in the implementation. `decode_deathscreen_legacy` returns a camera target flag and position.

## Client-to-server data

[`crate::protocol::player::PlayerPositionUpdate`] contains position, speed, pitch, yaw, pressed-key flags, FOV, wanted range, camera inversion, movement speed, and movement direction. `encode_player_position` serializes this state. The current implementation scales positional and angular values before writing integer fields and rejects invalid or out-of-range values.

`encode_interact` combines an [`crate::protocol::player::InteractAction`], wield index, [`crate::protocol::player::PointedThing`], and player update. `PointedThing` supports no target, a node target with under/above positions, or an object ID. `encode_player_item` selects a wield index.
