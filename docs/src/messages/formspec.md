# Forms and Formspec

Forms-related payload codecs and the formspec parser are in [`crate::protocol::formspec`]. The module handles the transport representation of forms and submitted fields; it is not a complete formspec renderer.

## Server updates

The module decodes inventory fields, inventory formspec updates, shown forms, detached inventory updates, and formspec prepend data. Their models are [`crate::protocol::formspec::InventoryFields`], [`crate::protocol::formspec::InventoryFormspec`], [`crate::protocol::formspec::ShowFormspec`], [`crate::protocol::formspec::DetachedInventoryUpdate`], and [`crate::protocol::formspec::FormspecPrepend`] in `protocol::formspec`.

## Parsed document

[`crate::protocol::formspec::FormspecDocument::parse`] parses a source string into `FormspecElement` values. Each element contains a name and a vector of arguments. `elements_named` provides iteration over elements matching a name.

## Client submissions

`encode_inventory_fields` and `encode_node_metadata_fields` serialize submitted field values. The module exposes `MAX_FORMSPEC_BYTES`, derived from the split-packet payload limit in the constants module.

Forms API version is declared separately in [`crate::protocol::constant::FORMSPEC_API_VERSION`]. It should not be confused with the network protocol version.
