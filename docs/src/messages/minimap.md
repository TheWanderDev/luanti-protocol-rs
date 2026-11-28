# Minimap Modes

`protocol::messages::to_client::MinimapModesUpdate` represents a server update containing the
available minimap modes and selected mode index. It implements `DecodeMessage`.

Decode the complete application message, including its command ID:

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{DecodeMessage, to_client};

fn decode_modes(data: &[u8]) -> Result<to_client::MinimapModesUpdate, ProtocolError> {
    to_client::MinimapModesUpdate::decode(data)
}
```

The underlying `decode_minimap_modes` function is crate-private. The public entry point is the
message type above. Texture loading, mode selection, and rendering belong to the client.
