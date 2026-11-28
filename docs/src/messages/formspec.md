# Forms and Formspec

Form-related network messages are exposed through typed message wrappers.

## Server-to-client messages

`protocol::messages::to_client` exports `InventoryFormspec`, `ShowFormspec`,
`DetachedInventoryUpdate`, and `FormspecPrepend`. These types implement `DecodeMessage` and consume
a complete application message, including the command ID.

## Client-to-server messages

`protocol::messages::to_server` exports `InventoryFields` and `NodeMetadataFields`. They implement
`EncodeMessage` and serialize submitted field values as complete application messages.

## Parsed formspec documents

`protocol::formspec::FormspecDocument::parse` parses a formspec source string into elements.
`elements_named` iterates over elements with a matching name. Parsing is not rendering: layout,
textures, interaction, and UI presentation belong to the client.

The formspec API version is declared separately in
`protocol::constant::FORMSPEC_API_VERSION`; it is not the network protocol version.
