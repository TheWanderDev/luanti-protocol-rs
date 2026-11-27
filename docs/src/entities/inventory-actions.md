# Inventory Actions

Outbound inventory actions are encoded by [`crate::protocol::inventory_action::encode_inventory_action`]. The action model is [`crate::protocol::inventory_action::InventoryAction`].

Supported variants are `Move`, `MoveSomewhere`, `Drop`, and `Craft`. Inventory locations are represented by [`crate::protocol::inventory_action::InventoryLocation`]: current player, named player, node metadata at an integer position, or detached inventory.

The encoder serializes the action as the inventory action text format and prefixes the `TOSERVER_INVENTORY_ACTION` command ID. It validates location/list tokens and rejects negative indices for operations that carry indices. A successful encoding only means the local representation was serialized; the server remains authoritative for inventory permissions, available counts, and action results.
