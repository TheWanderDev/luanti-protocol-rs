# Protocol Versions

The current source advertises network protocol versions **37–55**, serialization version **29**,
and formspec API version **12**. These values are separate compatibility dimensions.

| Value | Current setting | Meaning |
|---|---:|---|
| Minimum protocol | 37 | Lowest network protocol accepted by this source |
| Maximum protocol | 55 | Highest network protocol advertised by this source |
| Serialization version | 29 | Map and node serialization format version |
| Formspec API | 12 | Forms API capability reported by the client |

The constants are defined in `protocol::constant`. The selected protocol version is negotiated
with the server during the handshake; version-aware message decoders must receive that negotiated
value rather than the maximum supported value.

Protocol 55 adds the `move_camera` attachment flag to generic active-object `ATTACH_TO` data. The
flag byte uses `0x01` for `visible_on_attach` and `0x02` for `move_camera`. Older protocol versions
retain the visibility-only flag semantics.
