# Packet Framing

The MTP codec is implemented in [`crate::protocol::packet`]. Its base header is seven bytes:

| Offset | Width | Field |
|---:|---:|---|
| 0 | 4 | Protocol identifier (`u32`) |
| 4 | 2 | Sender peer ID (`u16`) |
| 6 | 1 | Channel (`u8`) |

All multi-byte fields are big-endian. The protocol identifier is [`crate::protocol::constant::PROTOCOL_ID`]. The current constants declare three channels and a maximum datagram size of 512 bytes.

## Body types

| Type | Value | Body |
|---|---:|---|
| Control | 0 | Control subtype and optional `u16` value |
| Original | 1 | Game payload bytes |
| Split | 2 | Split sequence, chunk count, chunk index, chunk bytes |
| Reliable | 3 | Reliable sequence followed by one nested body |

Control subtypes are ACK, SetPeerId, Ping, and Disconnect. The decoder rejects unknown packet/control types, invalid channels, malformed lengths, invalid split indices, nested reliable packets, and datagrams larger than the declared packet limit.

[`crate::protocol::packet::decode_datagram`] returns [`crate::protocol::packet::DecodedDatagram`]. It exposes a split chunk as data; it does not collect chunks into a complete application message or implement retransmission and ACK state.

## Encoders

The module provides `encode_header`, `encode_original`, `encode_reliable`, `encode_reliable_split`, `encode_ack`, `encode_set_peer_id`, `encode_ping`, and `encode_disconnect`. These functions serialize individual datagrams. The caller must supply valid peer, channel, sequence, and split metadata.
