# Mod Channels

Mod-channel messages and signals are implemented in [`crate::protocol::mod_channel`].

`ModChannelMessage` contains the channel name, sender name, and message. `decode_mod_channel_message` reads three `string16` byte fields and converts them using lossy UTF-8 decoding. It rejects trailing bytes.

`decode_mod_channel_signal` reads a signal code and channel name. Known signals are join success/failure, leave success/failure, channel-not-registered, and access-state update. Access states are `Initializing`, `ReadWrite`, and `ReadOnly`. Unknown signal codes or access-state values return `Ok(None)` rather than a protocol error; callers must handle that case.

Outbound functions are `encode_mod_channel_join`, `encode_mod_channel_leave`, and `encode_mod_channel_message`. They prepend the corresponding client command ID and use `string16` fields.
