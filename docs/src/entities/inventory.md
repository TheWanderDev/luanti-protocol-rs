# Inventory

Server inventory updates are exposed as `protocol::messages::to_client::InventoryUpdate` and
implement `DecodeMessageWithProtocolVersion`. Pass the negotiated protocol version when decoding.

Client inventory actions are exposed as `protocol::messages::to_server::InventoryAction` and
implement `EncodeMessage`.

The inventory data models and action models remain available from `protocol::inventory` and
`protocol::inventory_action`. Command-level message wrappers are the preferred entry points when
handling network application messages because they validate the command ID.
