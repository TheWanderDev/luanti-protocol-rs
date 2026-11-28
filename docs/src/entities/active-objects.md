# Active Objects

Active-object network updates are exposed through `protocol::messages::to_client`.

- `ActiveObjectRemoveAdd` decodes object additions and removals.
- `ActiveObjectMessagesUpdate` decodes updates addressed to active-object IDs.

The associated payload models and standalone codecs are defined in `protocol::active_object`.
Generic-object state commands include attachment, animation, bone, property, and texture updates.

## Protocol 55 attachment flags

`GenericActiveObjectAttachment` exposes `visible_on_attach` and `move_camera`. The attachment
flags byte uses bit `0x01` for visibility and bit `0x02` for camera movement. Protocol 37–54 retain
the visibility-only interpretation; the decoder extracts the two flags independently.

The crate parses attachment state but does not manage scene hierarchy, camera transforms, or
rendering.
