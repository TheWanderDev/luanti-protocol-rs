# Upstream Compatibility

This crate implements the binary application protocol and MTP datagram framing. Compatibility
claims are tied to the constants and decoders in the current source, not to comments copied from
upstream documentation.

## Advertised versions

- Network protocol: 37–55.
- Serialization version: 29.
- Formspec API version: 12.
- Client metadata: 5.19.0-dev.

## Protocol 55

The protocol 55 attachment extension adds the `MOVE_CAMERA` bit to the flags byte in the generic
active-object `ATTACH_TO` payload. `0x01` represents force visibility on attach and `0x02`
represents camera movement. The payload framing and flag-byte position remain unchanged. The
`GenericActiveObjectAttachment` model exposes the decoded values as `visible_on_attach` and
`move_camera`.

## Compatibility discipline

When updating protocol support, inspect the actual upstream sender and receiver implementations
for the affected command, then compare the exact byte layout with the local encoder and decoder.
Do not infer wire behavior from comments alone. Update the version constants only in the same
change that implements and reviews the corresponding wire behavior.
