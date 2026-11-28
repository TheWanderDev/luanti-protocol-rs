# Error Handling

All payload and packet decoders use [`crate::ProtocolError`] for structured failures. The enum distinguishes truncated data, oversized packets, invalid channels and protocol identifiers, unsupported packet/control types, unexpected command IDs, invalid string lengths, invalid handshake/authentication data, malformed split/reliable packets, and domain-specific failures.

Domain variants cover chat, player values, sound, map blocks, node definitions, item definitions, inventory, media, HUD, active objects, particles, mod channels, minimap, and formspec. `InvalidNodeDefContext` carries the content ID, node name, and reason to help locate a malformed definition.

The decompression module is an exception: its public functions return `Result<_, &'static str>`. Callers that integrate decompression with a payload decoder should map those messages into an appropriate domain error.

Treat decoder errors as malformed or unsupported input. Do not continue parsing a payload after an error, and do not silently substitute default values for missing required fields.
