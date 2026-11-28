# Architecture

The crate separates datagram framing, application-message dispatch, payload models, and standalone
binary codecs.

1. `protocol::packet` handles MTP datagram framing.
2. `protocol::messages` handles complete application messages and their command identifiers.
3. `protocol::messages::to_client` and `to_server` group message types by network direction.
4. Domain modules such as `active_object`, `inventory`, `mapblock`, and `server_state` define
   payload models and lower-level codecs used by message wrappers.
5. `ProtocolError` represents malformed, truncated, unsupported, or inconsistent protocol input.

Message codecs consume application messages, not MTP datagrams. Transport sequencing, reliability,
acknowledgements, retransmission, and split-message reassembly belong to the networking layer.
