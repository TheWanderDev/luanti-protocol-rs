# Protocol Versions

The uploaded source snapshot declares these values in [`crate::protocol::constant`]:

| Constant | Snapshot value | Meaning in this crate |
|---|---:|---|
| `MIN_PROTOCOL_VERSION` | 37 | Lowest protocol version declared by this client codec |
| `MAX_PROTOCOL_VERSION` | 54 | Highest protocol version declared by this client codec |
| `SERIALIZATION_VERSION` | 29 | Serialization version advertised by the snapshot |
| `FORMSPEC_API_VERSION` | 12 | Forms API version advertised by the snapshot |
| `ZSTD_PROTOCOL_VERSION` | 48 | Compression selection threshold in the compression module |
| `CLIENT_VERSION_MAJOR` | 5 | Client version major component |
| `CLIENT_VERSION_MINOR` | 18 | Client version minor component |
| `CLIENT_VERSION_PATCH` | 0 | Client version patch component |

The negotiated protocol version is a session value selected during the handshake. It is not necessarily equal to `MAX_PROTOCOL_VERSION`. Version-aware decoders must receive the negotiated value.

These are source-snapshot declarations, not a claim that every version boundary has been independently verified against the current upstream send/receive implementation. See [Upstream Compatibility](upstream-compatibility.md).
