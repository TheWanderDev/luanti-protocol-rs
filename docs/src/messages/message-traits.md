# Typed Application Messages

The `protocol::messages` module exposes complete application messages grouped by direction:
`to_server` contains client-to-server messages and `to_client` contains server-to-client messages.

## Message contract

- `Message` associates a type with its `COMMAND_ID`. Its `decode_payload` method validates the
  two-byte command ID and returns the remaining application payload.
- `EncodeMessage::encode` serializes the command ID followed by the message payload.
- `DecodeMessage::decode` parses a complete application message whose layout does not need an
  explicit negotiated version.
- `DecodeMessageWithProtocolVersion::decode` accepts the negotiated protocol version.
- `DecodeMessageWithSerializationVersion::decode` accepts the negotiated serialization version.

All message decoder inputs include the command ID. They must not receive an MTP datagram or a
payload slice with the command ID already removed.

## Decode a server message

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{DecodeMessage, to_client};

fn decode_hello(data: &[u8]) -> Result<to_client::Hello, ProtocolError> {
    to_client::Hello::decode(data)
}
```

## Decode a version-dependent message

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{DecodeMessageWithProtocolVersion, to_client};

fn decode_node_definitions(
    data: &[u8],
    protocol_version: u16,
) -> Result<to_client::NodeDefinitionsUpdate, ProtocolError> {
    to_client::NodeDefinitionsUpdate::decode(data, protocol_version)
}
```

## Encode a client message

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{EncodeMessage, to_server};

fn encode_chat() -> Result<bytes::Bytes, ProtocolError> {
    to_server::ChatMessage {
        message: String::from("Hello"),
    }
    .encode()
}
```

The returned `Bytes` contains an application message, not an MTP datagram. Add transport framing
in the networking layer.

## Dispatch

Use `protocol::messages::command_id` to read the command identifier from a complete application
message, then dispatch to the corresponding public type in `to_client` or `to_server`. Preserve
the complete message bytes when calling its `decode` method.
