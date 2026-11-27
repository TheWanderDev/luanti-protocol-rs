# Upstream Compatibility

Compatibility claims must be tied to the source revision being checked. The official upstream network-protocol guide describes the high-level command protocol and low-level MTP framing, and explicitly points implementers to the actual send paths in `client.cpp` and `server.cpp` and receive paths in `clientpackethandler.cpp` and `serverpackethandler.cpp` rather than relying only on the protocol header: [Network Protocol](https://docs.luanti.org/for-engine-devs/network-protocol/).

The current uploaded crate snapshot identifies itself as client version 5.18.0 and declares protocol versions 37–54, serialization version 29, and formspec API version 12. 
## Verification method

For each command family, compare all of the following against the same upstream revision:

1. The command ID and documented payload fields.
2. The client/server send function that writes the payload.
3. The opposite-side packet handler that reads it.
4. Version guards surrounding optional fields or alternate layouts.
5. The low-level connection implementation for packet type, channel, reliability, split, and ACK behavior.

A matching command number alone is not sufficient evidence of payload compatibility. Likewise, a decoder that accepts a payload does not establish that the server currently sends that form.

## Current verification status

The source archive supplied for this review contains the Rust codec implementation but not an upstream source checkout or pinned upstream commit. Therefore, this document records the crate's declared values and the official upstream verification method; it does not label every decoder as fully upstream-verified. Record the exact upstream commit and comparison evidence here when that checkout is available.
