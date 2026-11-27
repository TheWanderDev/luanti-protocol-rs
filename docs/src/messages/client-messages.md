# Client Messages

Client-to-server payload encoders are distributed by message family rather than collected into one dispatcher.

## Player and interaction

[`crate::protocol::player`] provides `encode_player_position`, `encode_interact`, and `encode_player_item`. Interaction data uses [`crate::protocol::player::InteractAction`] and [`crate::protocol::player::PointedThing`]. Position input is represented by [`crate::protocol::player::PlayerPositionUpdate`].

## Other client commands

[`crate::protocol::client_commands`] provides encoders for received/deleted block acknowledgements, removed sound IDs, damage, legacy respawn, and client display information. The display-information fields are grouped in [`crate::protocol::client_commands::ClientInfoUpdate`].

[`crate::protocol::chat::encode_chat_message`] encodes chat text. [`crate::protocol::inventory_action::encode_inventory_action`] encodes inventory actions. [`crate::protocol::media`] provides media request and token acknowledgement encoders. [`crate::protocol::mod_channel`] provides channel join, leave, and message encoders.

Each encoder returns `Bytes` or `Result<Bytes, ProtocolError>`. A successful result is the game message payload, including its command ID; the transport layer must add MTP framing separately.
