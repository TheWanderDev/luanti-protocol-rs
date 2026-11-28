# Item Definitions

Item-definition models and serialization helpers are in `protocol::itemdef`.
`ItemDefinition` represents server-provided item properties; supporting types cover images, tile
animation, tool capabilities, group capabilities, sound specifications, pointability, and wear-bar
parameters. `ItemRegistry` provides lookup and registration behavior, and
`DecodedItemDefinitions` is the decoded definitions-and-aliases result type.

For network input, use `protocol::messages::to_client::ItemDefinitionsUpdate`. It implements
`DecodeMessageWithProtocolVersion` and exposes the decoded definitions and aliases. The underlying
`decode_item_def` function is crate-private. `encode_item_def` remains a public standalone encoder.

The decoder validates protocol-dependent data. Rendering and gameplay behavior for decoded item
properties belong to the client layer.
