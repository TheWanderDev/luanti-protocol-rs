//! Internal entry points used by the cargo-fuzz targets.

const MAX_INPUT_SIZE: usize = 1_048_576;

fn accepts_input(data: &[u8]) -> bool {
    data.len() <= MAX_INPUT_SIZE
}

pub fn active_objects(data: &[u8]) {
    if !accepts_input(data) {
        return;
    }

    use crate::protocol::active_object::{
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
}

pub fn definitions(data: &[u8]) {
    if !accepts_input(data) {
        return;
    }

    use crate::protocol::{itemdef, nodedef, tiledef};

    for version in 37..=54 {
        let _ = itemdef::decode_item_def(data, version);
        let _ = nodedef::decode_node_def(data, version);
    }

    let _ = tiledef::decode_content_features_tiles(13, data);
}

pub fn server_payloads(data: &[u8]) {
    if !accepts_input(data) {
        return;
    }

    use crate::protocol::{
        chat, formspec, hud, media, minimap, mod_channel, player, server_state, status,
    };

    let _ = chat::decode_chat_message(data);
    let _ = formspec::decode_inventory_fields_payload(data);
    let _ = formspec::decode_inventory_formspec(data);
    let _ = formspec::decode_show_formspec(data);
    let _ = formspec::decode_detached_inventory(data);
    let _ = formspec::decode_formspec_prepend(data);
    let _ = hud::decode_hud_remove(data);
    let _ = minimap::decode_minimap_modes(data);
    let _ = mod_channel::decode_mod_channel_message(data);
    let _ = mod_channel::decode_mod_channel_signal(data);
    let _ = player::decode_deathscreen_legacy(data);
    let _ = player::decode_move_player(data);
    let _ = status::decode_movement_parameters(data);
    let _ = status::decode_player_breath(data);
    let _ = status::decode_player_hp(data);
    let _ = status::decode_player_speed(data);
    let _ = status::decode_time_of_day(data);
    let _ = server_state::decode_lighting(data);
    let _ = server_state::decode_cloud_params(data);
    let _ = server_state::decode_day_night_ratio(data);
    let _ = server_state::decode_eye_offset(data);
    let _ = server_state::decode_fade_sound(data);
    let _ = server_state::decode_fov(data);
    let _ = server_state::decode_hud_flags(data);
    let _ = server_state::decode_hud_param(data);
    let _ = server_state::decode_delete_particle_spawner(data);
    let _ = server_state::decode_moon(data);
    let _ = server_state::decode_play_sound(data);
    let _ = server_state::decode_player_list(data);
    let _ = server_state::decode_privileges(data);
    let _ = server_state::decode_relative_player_move(data);
    let _ = server_state::decode_stars(data);
    let _ = server_state::decode_stop_sound(data);
    let _ = server_state::decode_sun(data);
    let _ = server_state::decode_camera_mode(data);
    let _ = server_state::decode_csm_restriction_flags(data);

    for version in 37..=54 {
        let _ = hud::decode_hud_add(data, version);
        let _ = hud::decode_hud_change(data, version);
        let _ = server_state::decode_sky(data, version);
        let _ = server_state::decode_local_player_animations(data, version);
        let _ = media::decode_media_push(data, version);
        let _ = media::decode_announce_media(data, version);
        let _ = media::decode_media_bunch(data, version);
    }
}

pub fn world_data(data: &[u8]) {
    if !accepts_input(data) {
        return;
    }

    use crate::protocol::{inventory, mapblock, node_metadata, particle};

    for version in 27..=29 {
        let _ = mapblock::decode_map_block(data, version);
        let _ = mapblock::decode_add_node(data, version);
    }

    for version in 37..=54 {
        let _ = inventory::decode_inventory(data, version);
        let _ = particle::decode_add_particle_spawner(data, version);
    }

    let _ = mapblock::decode_remove_node(data);
    let _ = node_metadata::decode_node_metadata_list(data, false);
    let _ = node_metadata::decode_node_metadata_list(data, true);
    let _ = node_metadata::decode_node_metadata_changed(data);
    let _ = particle::decode_spawn_particle(data);
    let _ = particle::decode_spawn_particle_batch(data);
}
