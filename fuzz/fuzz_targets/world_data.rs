#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::fuzz::world_data;

fuzz_target!(|data: &[u8]| {
    world_data(data);
});
