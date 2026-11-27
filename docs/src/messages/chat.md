# Chat

Chat payload types and codecs are in [`crate::protocol::chat`]. [`crate::protocol::chat::ChatMessage`] contains a message type, sender, message body, and timestamp.

`ChatMessageType` has four wire values: `Raw`, `Normal`, `Announce`, and `System`. `decode_chat_message` requires message format version 1, reads the type, decodes sender and message as UTF-16BE strings, reads a `u64` timestamp, and rejects trailing bytes. Unknown versions and message types return dedicated protocol errors.

`encode_chat_message` creates a client chat command containing a wide string. It encodes the outbound text payload; it does not assign a sender name or timestamp.
