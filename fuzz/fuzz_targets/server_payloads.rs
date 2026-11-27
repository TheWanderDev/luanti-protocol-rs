#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::{
    chat::decode_chat_message,
    formspec::{
        decode_detached_inventory, decode_formspec_prepend, decode_inventory_fields,
        decode_inventory_formspec, decode_show_formspec,
    },
    hud::{decode_hud_add, decode_hud_change, decode_hud_remove},
    media::{decode_announce_media, decode_media_bunch, decode_media_push},
    minimap::decode_minimap_modes,
    mod_channel::{decode_mod_channel_message, decode_mod_channel_signal},
    player::{decode_deathscreen_legacy, decode_move_player},
    server_state::{
        decode_camera_mode, decode_cloud_params, decode_csm_restriction_flags,
        decode_day_night_ratio, decode_delete_particle_spawner, decode_eye_offset,
        decode_fade_sound, decode_fov, decode_hud_flags, decode_hud_param, decode_lighting,
        decode_local_player_animations, decode_moon, decode_play_sound, decode_player_list,
        decode_privileges, decode_relative_player_move, decode_sky, decode_stars, decode_stop_sound,
        decode_sun,
    },
    status::{
        decode_movement_parameters, decode_player_breath, decode_player_hp, decode_player_speed,
        decode_time_of_day,
    },
};

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }

    let _ = decode_chat_message(data);
    let _ = decode_inventory_fields(data);
    let _ = decode_inventory_formspec(data);
    let _ = decode_show_formspec(data);
    let _ = decode_detached_inventory(data);
    let _ = decode_formspec_prepend(data);
    let _ = decode_hud_remove(data);
    let _ = decode_minimap_modes(data);
    let _ = decode_mod_channel_message(data);
    let _ = decode_mod_channel_signal(data);
    let _ = decode_deathscreen_legacy(data);
    let _ = decode_move_player(data);
    let _ = decode_movement_parameters(data);
    let _ = decode_player_breath(data);
    let _ = decode_player_hp(data);
    let _ = decode_player_speed(data);
    let _ = decode_time_of_day(data);
    let _ = decode_lighting(data);
    let _ = decode_cloud_params(data);
    let _ = decode_day_night_ratio(data);
    let _ = decode_eye_offset(data);
    let _ = decode_fade_sound(data);
    let _ = decode_fov(data);
    let _ = decode_hud_flags(data);
    let _ = decode_hud_param(data);
    let _ = decode_delete_particle_spawner(data);
    let _ = decode_moon(data);
    let _ = decode_play_sound(data);
    let _ = decode_player_list(data);
    let _ = decode_privileges(data);
    let _ = decode_relative_player_move(data);
    let _ = decode_stars(data);
    let _ = decode_stop_sound(data);
    let _ = decode_sun(data);
    let _ = decode_camera_mode(data);
    let _ = decode_csm_restriction_flags(data);

    for version in 37..=54 {
        let _ = decode_hud_add(data, version);
        let _ = decode_hud_change(data, version);
        let _ = decode_sky(data, version);
        let _ = decode_local_player_animations(data, version);
        let _ = decode_media_push(data, version);
        let _ = decode_announce_media(data, version);
        let _ = decode_media_bunch(data, version);
    }
});
