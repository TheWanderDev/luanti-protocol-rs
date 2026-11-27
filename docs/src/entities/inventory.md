# Inventory

Inventory text and update structures are implemented in [`crate::protocol::inventory`]. The main data types are [`crate::protocol::inventory::ItemStack`], [`crate::protocol::inventory::InventoryList`], [`crate::protocol::inventory::Inventory`], and [`crate::protocol::inventory::InventoryUpdate`].

`ItemStack` models an item name, count, wear, and metadata representation as defined by the parser. Inventory lists hold named collections of stacks and their declared size; an inventory groups lists. `decode_inventory` parses the server inventory payload and accepts the protocol version because the wire representation changes across versions.

The module also contains parsing and formatting support for item-stack text and metadata. Consult the generated API reference for the exact public methods and fields; the text representation is not interchangeable with the binary packet framing layer.

This crate parses inventory state. It does not apply inventory actions to server state or predict whether a requested move/craft operation will be accepted.
