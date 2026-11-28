# Client Messages

Client-to-server application message types are exported from
`protocol::messages::to_server`. Import `EncodeMessage` and call `.encode()` to produce the
complete application message, including its two-byte command ID.

## Message families

| Family | Public message types |
|---|---|
| Handshake and authentication | `Init`, `Init2`, `FirstSrp`, `SrpBytesA`, `SrpBytesM`, `ClientReady` |
| Player and interaction | `PlayerPositionUpdate`, `PlayerItem`, `Interact` |
| Inventory and forms | `InventoryAction`, `InventoryFields`, `NodeMetadataFields` |
| Chat and channels | `ChatMessage`, `ModChannelJoin`, `ModChannelLeave`, `ModChannelMessage` |
| Media | `RequestMedia`, `HaveMedia` |
| Client commands | `GotBlocks`, `DeletedBlocks`, `RemovedSounds`, `Damage`, `RespawnLegacy`, `ClientInfoUpdate` |

## Example

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{EncodeMessage, to_server};

fn send_chat() -> Result<bytes::Bytes, ProtocolError> {
    to_server::ChatMessage {
        message: String::from("Hello"),
    }
    .encode()
}
```

`Bytes` contains the application message. MTP framing, sequencing, and transport delivery are
handled by the networking layer.
