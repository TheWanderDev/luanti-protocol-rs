# Wire Protocol

The implementation separates the low-level MTP datagram envelope from the high-level game command payload.

## Integer and string representation

The primitive reader and writer use big-endian byte order. The reader exposes unsigned and signed integer methods, `f32` as the bit pattern of a big-endian `u32`, and `f1000` as a signed 32-bit integer divided by 1000. Length-prefixed byte strings use a big-endian `u16` or `u32` length. Wide strings use a `u16` count of UTF-16 code units followed by big-endian UTF-16 units.

The concrete implementations are [`crate::protocol::reader::PacketReader`] and [`crate::protocol::writer::PacketWriter`].

## High-level command payload

A game message starts with a `u16` command identifier. The identifiers are represented by [`crate::protocol::commands::ToClientCommand`] and [`crate::protocol::commands::ToServerCommand`]. The remaining bytes are decoded by the module associated with that command.

## Version-dependent fields

Several decoders accept a negotiated protocol version or serialization version explicitly. That argument controls the payload layout in those implementations; callers must pass the version negotiated with the server, not merely the maximum supported version declared by the crate. Examples include media, HUD, active objects, map blocks, item definitions, and node definitions.