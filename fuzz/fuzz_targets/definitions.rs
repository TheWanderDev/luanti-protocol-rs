#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::{
    itemdef::decode_item_def,
    nodedef::decode_node_def,
    tiledef::decode_content_features_tiles,
};

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }

    for version in 37..=54 {
        let _ = decode_node_def(data, version);
        let _ = decode_item_def(data, version);
    }

    let _ = decode_content_features_tiles(13, data);
});
