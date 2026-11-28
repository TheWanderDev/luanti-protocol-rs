# Mod Channels

Mod-channel application messages are exposed through `protocol::messages`.

- `protocol::messages::to_client::ModChannelMessage` decodes messages received from the server.
- `protocol::messages::to_client::ModChannelSignal` decodes channel state notifications. Its
  `update` field is optional because unknown signal values are represented as `None`.
- `protocol::messages::to_server::ModChannelJoin`, `ModChannelLeave`, and `ModChannelMessage`
  encode client requests and messages.

Use `DecodeMessage` for inbound types and `EncodeMessage` for outbound types. The wrapper types
validate the application command ID; their lower-level payload helpers are implementation details.
