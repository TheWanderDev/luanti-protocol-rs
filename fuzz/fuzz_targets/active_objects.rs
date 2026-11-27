#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::active_object::{
    decode_active_object_messages, decode_active_object_remove_add,
    decode_generic_active_object_animation, decode_generic_active_object_animation_speed,
    decode_generic_active_object_attachment, decode_generic_active_object_armor_groups,
    decode_generic_active_object_bone_position, decode_generic_active_object_hp_update,
    decode_generic_active_object_infant, decode_generic_active_object_init,
    decode_generic_active_object_message, decode_generic_active_object_nametag_attributes,
    decode_generic_active_object_physics_override, decode_generic_active_object_position_update,
    decode_generic_active_object_sprite, decode_generic_active_object_stop_animation,
    decode_generic_active_object_texture_modifier, decode_generic_object_properties,
};

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }

    for version in 37..=54 {
        let _ = decode_generic_active_object_animation(data, version);
        let _ = decode_generic_active_object_animation_speed(data, version);
        let _ = decode_generic_active_object_stop_animation(data, version);
        let _ = decode_generic_active_object_infant(data, version);
    }

    let _ = decode_generic_active_object_armor_groups(data);
    let _ = decode_generic_active_object_bone_position(data);
    let _ = decode_generic_active_object_attachment(data);
    let _ = decode_generic_active_object_physics_override(data);
    let _ = decode_generic_active_object_texture_modifier(data);
    let _ = decode_generic_active_object_sprite(data);
    let _ = decode_generic_active_object_nametag_attributes(data);
    let _ = decode_generic_active_object_hp_update(data);
    let _ = decode_generic_active_object_position_update(data);
    let _ = decode_generic_object_properties(data);
    let _ = decode_generic_active_object_message(data);
    let _ = decode_generic_active_object_init(data);
    let _ = decode_active_object_remove_add(data);
    let _ = decode_active_object_messages(data);
});
