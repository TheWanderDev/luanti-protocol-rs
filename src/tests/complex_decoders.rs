use crate::protocol::active_object::{
    decode_active_object_messages, decode_active_object_remove_add,
    decode_generic_active_object_init, decode_generic_active_object_message,
};
use crate::protocol::formspec::{
    decode_detached_inventory, decode_formspec_prepend, decode_inventory_formspec,
    decode_show_formspec,
};
use crate::protocol::hud::{decode_hud_add, decode_hud_change, decode_hud_remove};
use crate::protocol::inventory::decode_inventory;
use crate::protocol::itemdef::decode_item_def;
use crate::protocol::mapblock::{decode_add_node, decode_map_block, decode_remove_node};
use crate::protocol::media::{decode_announce_media, decode_media_bunch};
use crate::protocol::node_metadata::{decode_node_metadata_changed, decode_node_metadata_list};
use crate::protocol::nodedef::decode_node_def;

#[test]
fn mapblock_and_node_payload_decoders_reject_empty_input() {
    assert!(decode_map_block(&[], 29).is_err());
    assert!(decode_add_node(&[], 29).is_err());
    assert!(decode_remove_node(&[]).is_err());
    assert!(decode_node_def(&[], 53).is_err());
    assert!(decode_item_def(&[], 53).is_err());
    assert!(decode_node_metadata_list(&[], false).is_err());
    assert!(decode_node_metadata_changed(&[]).is_err());
}

#[test]
fn inventory_media_and_hud_decoders_reject_empty_input() {
    assert!(decode_inventory(&[], 53).is_err());
    assert!(decode_announce_media(&[], 53).is_err());
    assert!(decode_media_bunch(&[], 53).is_err());
    assert!(decode_hud_add(&[], 53).is_err());
    assert!(decode_hud_remove(&[]).is_err());
    assert!(decode_hud_change(&[], 53).is_err());
}

#[test]
fn formspec_and_active_object_decoders_reject_empty_input() {
    assert!(decode_inventory_formspec(&[]).is_err());
    assert!(decode_show_formspec(&[]).is_err());
    assert!(decode_detached_inventory(&[]).is_err());
    assert!(decode_formspec_prepend(&[]).is_err());
    assert!(decode_generic_active_object_init(&[]).is_err());
    assert!(decode_generic_active_object_message(&[]).is_err());
    assert!(decode_active_object_remove_add(&[]).is_err());
    assert_eq!(decode_active_object_messages(&[]), Ok(Vec::new()));
}

/*next time maybe*/
