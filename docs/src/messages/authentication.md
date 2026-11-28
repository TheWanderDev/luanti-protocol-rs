# Connection and Authentication Messages

Handshake message types are grouped by direction. The crate serializes and parses protocol
messages; SRP proof calculation and cryptographic key exchange belong to the authentication layer.

## Client to server

Types are exported from `protocol::messages::to_server`:

- `Init` advertises the player's name, protocol range, and serialization version.
- `Init2` acknowledges the initial server handshake and carries the language string.
- `FirstSrp`, `SrpBytesA`, and `SrpBytesM` carry the corresponding first-SRP and SRP exchange data.
- `ClientReady` announces the client version and supported formspec API version.

These messages implement `EncodeMessage`; call `.encode()` to obtain the complete application
message, including its command ID.

## Server to client

Types are exported from `protocol::messages::to_client`:

- `Hello` contains the selected protocol version, serialization version, and authentication
  mechanism flags.
- `AuthAccept` contains the map seed, recommended send interval, and sudo authentication flags.
- `AccessDenied` and `AccessDeniedLegacy` represent denial responses.
- `AcceptSudoMode`, `DenySudoMode`, and `SrpChallenge` represent subsequent authentication state.

These message types implement the appropriate decode trait. Use `DecodeMessage::decode` for
messages without a separate version argument.

## Example

```rust,no_run
use luanti_protocol_rs::ProtocolError;
use luanti_protocol_rs::protocol::messages::{DecodeMessage, EncodeMessage, to_client, to_server};

fn make_init() -> Result<bytes::Bytes, ProtocolError> {
    to_server::Init {
        player_name: String::from("player"),
        protocol_min: 37,
        protocol_max: 55,
        serialization_version: 29,
    }
    .encode()
}

fn read_hello(data: &[u8]) -> Result<to_client::Hello, ProtocolError> {
    to_client::Hello::decode(data)
}
```

## Mechanism flags

Authentication mechanism constants are declared in `protocol::constant`. The client selects a
mechanism supported by both peers and performs the corresponding cryptographic exchange outside
this codec crate.
