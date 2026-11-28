# Protocol Codec

This book documents the Rust protocol codec crate: its MTP framing, binary readers and writers,
typed application messages, domain payloads, and compatibility rules.

## Start here

- [Architecture](fundamentals/architecture.md) explains the boundaries between transport framing
  and application messages.
- [Typed Application Messages](messages/message-traits.md) explains the public message traits and
  complete-message input contract.
- [AI-Oriented API Guide](ai-guide.md) provides a compact map of public entry points and message
  families.
- [Public API](reference/public-api.md) distinguishes supported public APIs from crate-private
  implementation helpers.
- [Protocol Versions](reference/protocol-versions.md) records the supported protocol and
  serialization versions.

## Message direction

Use `protocol::messages::to_server` for messages encoded by the client and
`protocol::messages::to_client` for messages decoded from the server. All message decoders receive
the complete application message, including the two-byte command ID.

## Scope

The crate does not provide sockets, peer/session management, reliable delivery, retransmission,
ACK scheduling, split-message reassembly, authentication cryptography, media downloads, or
rendering. Those responsibilities belong to higher layers.
