# luanti-protocol-rs

A Rust implementation of the client-server binary protocol, providing the protocol foundation for Rust-based server and client applications.

<p align="center">
  <a href="https://thewanderdev.github.io/luanti-protocol-rs/">
    <img src="https://img.shields.io/badge/Documentation-User_Guide-2563eb?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Documentation">
  </a>
  <a href="https://thewanderdev.github.io/luanti-protocol-rs/api/luanti_protocol_rs/">
    <img src="https://img.shields.io/badge/API_Reference-Rustdoc-orange?style=for-the-badge&logo=rust&logoColor=white" alt="API Reference">
  </a>
  <a href="https://crates.io/crates/luanti-protocol-rs">
    <img src="https://img.shields.io/crates/v/luanti-protocol-rs?style=for-the-badge&logo=rust&cacheSeconds=300" alt="Latest crates.io version">
  </a>
</p>

## Overview

**luanti-protocol-rs** implements the binary protocol used for communication between clients and servers.

It provides the shared protocol layer for:

- **luanti-runtime-rs** — server and client runtime components.
- **luanti-client-headless-rs** — headless client implementation.
- **luanti-client-gui-rs** — graphical client implementation.

The goal is to provide one reusable protocol implementation across these applications.

## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
luanti-protocol-rs = "0.5.19-pre"
```

## Public API

The crate exposes typed application messages through direction-specific modules at the crate root:

- `luanti_protocol_rs::to_server` contains messages sent by a client to a server.
- `luanti_protocol_rs::to_client` contains messages sent by a server to a client.

The direction-specific modules are also available through `luanti_protocol_rs::protocol::messages` for code using the previous module path.

Message traits remain under `luanti_protocol_rs::protocol::messages`:

- `EncodeMessage` encodes a complete application message.
- `DecodeMessage` decodes a complete application message and validates its command ID.
- `DecodeMessageWithProtocolVersion` handles messages whose payload depends on the negotiated protocol version.
- `DecodeMessageWithSerializationVersion` handles messages whose payload depends on the serialization version.

`ProtocolError` is available directly from the crate root. Lower-level modules such as `protocol::reader`, `protocol::writer`, and `protocol::packet` remain available for primitive fields and MTP datagram framing.

### Encode and decode a client message

```rust
use luanti_protocol_rs::protocol::messages::{DecodeMessage, EncodeMessage};
use luanti_protocol_rs::to_server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = to_server::ChatMessage {
        message: String::from("Hello server"),
    };

    // Encoded bytes include the two-byte command ID.
    let encoded = message.encode()?;

    // Decode the complete application message using its typed message API.
    let decoded = to_server::ChatMessage::decode(&encoded)?;
    assert_eq!(decoded, message);

    Ok(())
}
```

### Decode a server message

```rust
use luanti_protocol_rs::protocol::messages::DecodeMessage;
use luanti_protocol_rs::{to_client, ProtocolError};

fn decode_hello(data: &[u8]) -> Result<to_client::Hello, ProtocolError> {
    to_client::Hello::decode(data)
}
```

### Decode a version-dependent message

```rust
use luanti_protocol_rs::protocol::messages::DecodeMessageWithProtocolVersion;
use luanti_protocol_rs::{to_client, ProtocolError};

fn decode_inventory(
    data: &[u8],
    protocol_version: u16,
) -> Result<to_client::InventoryUpdate, ProtocolError> {
    to_client::InventoryUpdate::decode(data, protocol_version)
}
```

### Application messages and packet framing

Message codecs consume a complete application message, including its two-byte command ID. They do not consume an MTP datagram. Use `protocol::packet` for datagram framing. Transport sequencing, reliability, acknowledgements, and split-message reassembly belong to the networking layer.

The standalone payload codec functions used internally by typed messages are crate-private. Applications should use the typed message API instead of depending on internal payload decoder functions. This keeps implementation details separate from the supported public API.

## Documentation

| Resource | Description |
|---|---|
| [**User Guide**](https://thewanderdev.github.io/luanti-protocol-rs/) | Guides and technical documentation |
| [**API Reference**](https://thewanderdev.github.io/luanti-protocol-rs/api/luanti_protocol_rs/) | Rust API documentation |
| [**Crates.io**](https://crates.io/crates/luanti-protocol-rs) | Published Rust package |
| [**GitHub Repository**](https://github.com/TheWanderDev/luanti-protocol-rs) | Source code and development |

## License

Licensed under the [**LGPL-3.0-only**](../../LICENSE.md) license.
