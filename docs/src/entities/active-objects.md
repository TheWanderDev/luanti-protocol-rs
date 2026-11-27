# Active Objects

Active-object payload structures and decoders are in [`crate::protocol::active_object`]. The module represents object creation/removal batches, per-object messages, generic-object initialization, and the generic-object command stream.

`decode_active_object_remove_add` returns [`crate::protocol::active_object::ActiveObjectUpdate`] with removed IDs and added [`crate::protocol::active_object::ActiveObjectData`] entries. `decode_active_object_messages` decodes updates addressed to object IDs. `decode_generic_active_object_init` returns [`crate::protocol::active_object::GenericActiveObjectInit`], while `decode_generic_active_object_message` separates a command ID from its payload.

The generic object model includes properties, position updates, armor groups, animations, bone transforms, attachments, physics overrides, infant references, sprites, nametag attributes, and texture modifiers. `GenericActiveObjectState` collects the decoded state. `GenericActiveObjectCommandKind` and `GenericActiveObjectTrackId` describe command and animation-track categories.

Some command formats depend on protocol version. The source declares version thresholds for stop-animation and animation-track support. Pass the negotiated protocol version to the relevant decoders and retain unknown extension bytes where the model exposes them.
