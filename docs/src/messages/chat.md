# Chat

Chat application messages are exposed through the typed message API.

- `protocol::messages::to_client::ChatMessage` decodes server chat messages and also implements
  `EncodeMessage` for its wire representation.
- `protocol::messages::to_server::ChatMessage` encodes client chat text.

Import `DecodeMessage` or `EncodeMessage` as appropriate. The chat payload codec functions are
crate-private implementation details; downstream code should use these public message types.

Chat messages are application messages. MTP datagram framing and transport delivery are outside
this module.
