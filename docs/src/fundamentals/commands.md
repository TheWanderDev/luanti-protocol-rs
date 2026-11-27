# Commands and Opcodes

Command IDs are defined in [`crate::protocol::commands`]. The server-to-client direction uses [`crate::protocol::commands::ToClientCommand`]; the client-to-server direction uses [`crate::protocol::commands::ToServerCommand`]. Both enums use `u16` representations and provide numeric conversion through their derives.

## Client command metadata

[`crate::protocol::commands::ToServerCommand::channel`] returns the channel assigned by this crate for each outbound command. `is_reliable` returns `false` only for `Init` and `PlayerPosition`; all other variants return `true`.

These helpers describe the crate's command metadata. They do not implement the reliable transport, sequence counters, retransmission, or ACK processing.

## Command families

The server-to-client enum includes handshake and authentication, map updates, inventory, time and player status, media, chat, active objects, definitions, sound, HUD, formspec, particles, sky and lighting, mod channels, node metadata, and minimap updates.

The client-to-server enum includes initialization, mod-channel operations, player position and interaction, block acknowledgements, inventory actions, chat, damage, selected item, media requests, client readiness, SRP payloads, and client display information.

The `TOCLIENT_NUM_MSG_TYPES` and `TOSERVER_NUM_MSG_TYPES` constants are exposed as command-count metadata. Do not infer support for a command solely from these counts; use the enum variants and the payload codec that handles the command.
