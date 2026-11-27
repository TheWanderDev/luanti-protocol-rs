#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::{
    inventory::decode_inventory,
    mapblock::{decode_add_node, decode_map_block, decode_remove_node},
    node_metadata::{decode_node_metadata_changed, decode_node_metadata_list},
    particle::{decode_add_particle_spawner, decode_spawn_particle, decode_spawn_particle_batch},
};

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }

    for version in 27..=29 {
        let _ = decode_map_block(data, version);
        let _ = decode_add_node(data, version);
    }

    for version in 37..=54 {
        let _ = decode_inventory(data, version);
        let _ = decode_add_particle_spawner(data, version);
    }

    let _ = decode_remove_node(data);
    let _ = decode_node_metadata_list(data, false);
    let _ = decode_node_metadata_list(data, true);
    let _ = decode_node_metadata_changed(data);
    let _ = decode_spawn_particle(data);
    let _ = decode_spawn_particle_batch(data);
});
