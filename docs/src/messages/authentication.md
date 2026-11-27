# Connection and Authentication Messages

Handshake payload structures and selected encoders/decoders are in [`crate::protocol::messages`]. This module handles protocol message serialization; it does not calculate SRP proofs or perform cryptographic key exchange.

## Outbound messages

- `encode_init2(language)` serializes the second initialization message.
- `encode_first_srp` creates the first-SRP message payload.
- `encode_srp_bytes_a(public_a, legacy_password)` serializes the client public value and legacy-mode flag.
- `encode_srp_bytes_m(proof)` serializes the client proof.
- `encode_client_ready()` creates the client-ready message.

## Inbound messages

`decode_auth_accept_payload` returns [`crate::protocol::messages::AuthAcceptData`], containing the map seed, recommended send interval, and sudo authentication mechanisms. `decode_access_denied_payload`, `decode_sudo_mode_payload`, and `decode_sudo_mode_denial` decode their respective payloads. `decode_auth_server_message` reads the command ID and returns [`crate::protocol::messages::AuthServerMessage`].

The module also exposes [`crate::protocol::messages::InitMessage`] and [`crate::protocol::messages::HelloMessage`] as typed handshake data structures.

## Authentication mechanism flags

The bit flags are declared in [`crate::protocol::constant`]: legacy password, SRP, and first-SRP. The application layer selects a mechanism supported by both peers and performs the associated cryptographic exchange.
