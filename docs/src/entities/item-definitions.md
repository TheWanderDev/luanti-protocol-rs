# Item Definitions

Item definition structures and codecs are in [`crate::protocol::itemdef`]. [`crate::protocol::itemdef::ItemDefinition`] represents the server-provided item properties. Supporting types cover item images, tile animation, tool capabilities, group capabilities, sound specifications, pointability, wear-bar parameters, and registry lookup.

`ItemRegistry` stores definitions and provides the module's item lookup/registration behavior. `DecodedItemDefinitions` is the public tuple alias returned by the item-definition decoder path. `decode_item_def` parses item definitions; `encode_item_def` serializes them.

The decoder accepts protocol-version information where the payload layout is version-sensitive. The module contains a declared item type constant for tools. Do not infer that every rendering or gameplay property is consumed by this codec; the API describes decoded data, while application behavior belongs to the client layer.
