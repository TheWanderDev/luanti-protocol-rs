# Inventory Actions

Client inventory actions are represented by `protocol::messages::to_server::InventoryAction`.
The type is re-exported from `protocol::inventory_action` and implements `EncodeMessage`.

Use `EncodeMessage::encode` to serialize the complete client application message, including its
command ID. The lower-level `encode_inventory_action` helper is crate-private and should not be
used by downstream crates.

Inventory action models describe operations such as moving, dropping, and craft-related inventory
changes. The server remains responsible for validating and applying submitted actions.
