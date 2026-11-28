# Player State and Interaction

Player-related application messages are grouped by direction.

## Server-to-client

`protocol::messages::to_client` exports `ServerPlayerMovement`, `LegacyDeathscreen`,
`MovementParameters`, `TimeOfDayUpdate`, `HpUpdate`, `PlayerSpeedUpdate`, `PlayerBreathUpdate`, and
`LocalPlayerAnimationsUpdate`. Use each type's implemented decode trait.

## Client-to-server

`protocol::messages::to_server` exports `PlayerPositionUpdate`, `PlayerItem`, and `Interact`.
These types implement `EncodeMessage`. `Interact` combines an interaction action, wield index,
pointed target, and player movement state.

The public message wrappers encode or decode complete application messages. They do not update a
local player simulation or render player state.
