# Commands and Opcodes

Application command identifiers are two-byte values at the start of each complete application
message. `protocol::messages::command_id` reads this identifier from a complete message.

Each typed message implements `Message` and declares its identifier through `COMMAND_ID`.
`Message::decode_payload` validates that identifier before a decoder parses the remaining payload.
Use the matching public type in `protocol::messages::to_client` or `to_server` rather than calling
crate-private payload helpers.

MTP packet headers and transport control fields are a separate layer handled by
`protocol::packet`.
